use std::collections::HashMap;

use std::rc::Rc;
use std::cell::RefCell;

use ecs::Entity;

use nc::partitioning::{DBVT, DBVTLeaf};
use nc::partitioning::BoundingVolumeInterferencesCollector;
use nc::bounding_volume::AABB2;
use nc::bounding_volume::BoundingVolume;

use na::Pnt2;
use na::Vec2;
use na::Translation;

use components::Collision;
use components::Position;
use components::CollisionType;

use ordered_float::NotNaN;

type CollisionTreeLeaf = Rc<RefCell<DBVTLeaf<Pnt2<f32>, Entity, AABB2<f32>>>>;

struct CollisionTreeLeafs {
    x: CollisionTreeLeaf,
    y: CollisionTreeLeaf,
    coll_type: CollisionType,
}

pub struct CollisionWorld {
    dbvt_x: DBVT<Pnt2<f32>, Entity, AABB2<f32>>,
    dbvt_y: DBVT<Pnt2<f32>, Entity, AABB2<f32>>,
    mapping: HashMap<Entity, CollisionTreeLeafs>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Axis {
    X,
    Y,
}

fn find_depth(dyn: &AABB2<f32>, dyn_last: &Pnt2<f32>, stat: &AABB2<f32>, axis: Axis) -> Option<f32> {
    use self::Axis::{X, Y};

    if !dyn.intersects(stat) {
        return None;
    }

    let min_dist = match axis {
        X => dyn.half_extents().x + stat.half_extents().x,
        Y => dyn.half_extents().y + stat.half_extents().y,
    };

    let dist = match axis {
        X => dyn.center().x - stat.center().x,
        Y => dyn.center().y - stat.center().y,
    }.abs();

    let depth = min_dist - dist;

    if depth <= 0.0 {
        return None;
    }

    let dir = match axis {
        X => dyn_last.x - stat.center().x,
        Y => dyn_last.y - stat.center().y,
    }.signum();

    Some(dir * depth)
}

impl CollisionWorld {
    pub fn new() -> CollisionWorld {
        CollisionWorld {
            dbvt_x: DBVT::new(),
            dbvt_y: DBVT::new(),
            mapping: HashMap::new(),
        }
    }

    pub fn add(&mut self, e: Entity, coll: &Collision, pos: &Position) {
        // if it already existed, just remove it
        self.remove(e);

        // then add leafs for both trees
        self.mapping.insert(e, CollisionTreeLeafs {
            x: self.dbvt_x.insert_new(e, coll.aabb_x(Vec2::new(pos.x, pos.y))),
            y: self.dbvt_y.insert_new(e, coll.aabb_y(Vec2::new(pos.x, pos.y))),
            coll_type: coll.collision_type(),
        });
    }

    // move an entity along one exis, return collision depth if a collision occured
    fn move_axis(&mut self, leafs: &mut CollisionTreeLeafs, coll: &Collision, new_pos: &Position, last_pos: &Position, axis: Axis) -> Option<f32> {
        let aabb = match axis {
            Axis::X => coll.aabb_x(new_pos.as_vec()).merged(&coll.aabb_x(last_pos.as_vec())),
            Axis::Y => coll.aabb_y(new_pos.as_vec()).merged(&coll.aabb_y(last_pos.as_vec())),
        };

        let leaf = match axis {
            Axis::X => leafs.x.borrow(),
            Axis::Y => leafs.y.borrow(),
        };

        // find closest colliding entity
        let mut colls: Vec<Entity> = Vec::new();
        let closest: Option<&Entity> = match axis {
            Axis::X => {
                self.dbvt_x.visit(&mut BoundingVolumeInterferencesCollector::new(&aabb, &mut colls));

                let center_x = leaf.center;

                colls.iter().min_by_key(|other| {
                    let other_leafs = self.mapping.get(other).unwrap();
                    NotNaN::new((other_leafs.x.borrow().center.x - center_x.x).abs()).unwrap()
                })
            },
            Axis::Y => {
                self.dbvt_y.visit(&mut BoundingVolumeInterferencesCollector::new(&aabb, &mut colls));

                let center_y = leaf.center;

                colls.iter().min_by_key(|other| {
                    let other_leafs = self.mapping.get(other).unwrap();
                    NotNaN::new((other_leafs.y.borrow().center.y - center_y.y).abs()).unwrap()
                })
            }
        };

        // if one was found, test again, this time only in one direction (dbvt tests in both)
        if let Some(other) = closest {
            let other_leafs = self.mapping.get(other).unwrap();

            if leafs.coll_type != CollisionType::Solid ||
                other_leafs.coll_type != CollisionType::Solid {
                    // TODO fire event, return event, etc.?
                    return None;
                }

            let other_leaf = match axis {
                Axis::X => other_leafs.x.borrow(),
                Axis::Y => other_leafs.y.borrow(),
            };

            return find_depth(&aabb, &leaf.center, &other_leaf.bounding_volume, axis);
        };

        return None;
    }

    pub fn move_entity(&mut self, e: Entity, coll: &Collision, new_pos: &Position, last_pos: &Position) -> Position {
        // 1. remove both leafs
        let mut leafs: CollisionTreeLeafs = self.mapping.remove(&e).unwrap();

        self.dbvt_x.remove(&mut leafs.x);
        self.dbvt_y.remove(&mut leafs.y);

        let mut updated_pos = new_pos.clone();

        // 2. call move_axis for both axes, X first
        if let Some(depth_x) = self.move_axis(&mut leafs, coll, &updated_pos, last_pos, Axis::X) {
            let mut lx = leafs.x.borrow_mut();
            updated_pos.x += depth_x;

            lx.center.x = lx.center.x + depth_x;
            lx.bounding_volume.append_translation_mut(&Vec2::new(depth_x, 0.0));
        }

        if let Some(depth_y) = self.move_axis(&mut leafs, coll, &updated_pos, last_pos, Axis::Y) {
            let mut ly = leafs.y.borrow_mut();
            updated_pos.y += depth_y;

            ly.center.y = ly.center.y + depth_y;
            ly.bounding_volume.append_translation_mut(&Vec2::new(0.0, depth_y));
        }

        // 3. re-insert into trees
        self.dbvt_x.insert(leafs.x.clone());
        self.dbvt_y.insert(leafs.y.clone());

        // 4. update mapping
        self.mapping.insert(e, leafs);

        // return new position after collisions have been resolved
        return updated_pos;
    }

    pub fn remove(&mut self, e: Entity) {
        let mut leafs = match self.mapping.remove(&e) {
            None => return,
            Some(l) => l,
        };

        self.dbvt_x.remove(&mut leafs.x);
        self.dbvt_y.remove(&mut leafs.y);
    }
}
