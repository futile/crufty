use std::collections::HashMap;
use std::collections::hash_map::Entry;

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
use num::traits::Zero;

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
    dbvt: DBVT<Pnt2<f32>, Entity, AABB2<f32>>,
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
            dbvt: DBVT::new(),
            mapping: HashMap::new(),
        }
    }

    pub fn add(&mut self, e: Entity, coll: &Collision, pos: &Position) {
        match self.mapping.entry(e) {
            Entry::Occupied(_) => panic!("adding entity to CollisionWorld twice"),
            Entry::Vacant(v) => v.insert(CollisionTreeLeafs {
                x: self.dbvt.insert_new(e, coll.aabb_x(Vec2::new(pos.x, pos.y))),
                y: self.dbvt.insert_new(e, coll.aabb_y(Vec2::new(pos.x, pos.y))),
                coll_type: coll.collision_type(),
            }),
        };
    }

    pub fn move_entity(&mut self, e: Entity, coll: &Collision, new_pos: &Position, last_pos: &Position) -> Position {
        // 1. remove both leafs
        let mut leafs: CollisionTreeLeafs = self.mapping.remove(&e).unwrap();

        self.dbvt.remove(&mut leafs.x);
        self.dbvt.remove(&mut leafs.y);

        // 2. get new x-aabb
        let aabb_x = coll.aabb_x(new_pos.as_vec()).merged(&coll.aabb_x(last_pos.as_vec()));

        // 3. test for interferences -> resolve
        let mut colls: Vec<Entity> = Vec::new();
        self.dbvt.visit(&mut BoundingVolumeInterferencesCollector::new(&aabb_x, &mut colls));

        let center_x = leafs.x.borrow().center;
        let closest = colls.iter().min_by_key(|other| {
            let other_leafs = self.mapping.get(other).unwrap();
            NotNaN::new((other_leafs.x.borrow().center.x - center_x.x).abs()).unwrap()
        });

        let mut moved = Vec2::new(0.0f32, 0.0);

        // 3.1 find closest x-interference -> resolve
        if let Some(other) = closest {
            let other_leafs = self.mapping.get(other).unwrap();

            if leafs.coll_type != CollisionType::Solid ||
               other_leafs.coll_type != CollisionType::Solid {
                // TODO fire event ect?
               } else {
                   let depth_x = find_depth(&leafs.x.borrow().bounding_volume, &last_pos.as_pnt(), &other_leafs.x.borrow().bounding_volume, Axis::X).unwrap();
                   moved.x += depth_x;
               }
        } else {
            // 4. get new y-aabb
            let aabb_y = coll.aabb_y(new_pos.as_vec()).merged(&coll.aabb_y(last_pos.as_vec()));

            // 5. test for interferences -> resolve
            let mut colls: Vec<Entity> = Vec::new();
            self.dbvt.visit(&mut BoundingVolumeInterferencesCollector::new(&aabb_y, &mut colls));

            let center_y = leafs.y.borrow().center;
            let closest = colls.iter().min_by_key(|other| {
                let other_leafs = self.mapping.get(other).unwrap();
                NotNaN::new((other_leafs.y.borrow().center.y - center_y.y).abs()).unwrap()
            });

            if let Some(other) = closest {
                let other_leafs = self.mapping.get(other).unwrap();

                if leafs.coll_type != CollisionType::Solid ||
                    other_leafs.coll_type != CollisionType::Solid {
                        // TODO fire event ect?
                    } else {
                        let depth_y = find_depth(&leafs.y.borrow().bounding_volume, &last_pos.as_pnt(), &other_leafs.y.borrow().bounding_volume, Axis::Y).unwrap();
                        moved.y += depth_y;
                    }
            }
        }

        // 6. update leafs
        if !moved.is_zero() {
            let mut lx = leafs.x.borrow_mut();
            let mut ly = leafs.y.borrow_mut();

            lx.center = lx.center + moved;
            lx.bounding_volume.append_translation_mut(&moved);

            ly.center = ly.center + moved;
            ly.bounding_volume.append_translation_mut(&moved);
        }

        // 7. insert into tree
        self.dbvt.insert(leafs.x.clone());
        self.dbvt.insert(leafs.y.clone());

        // 8. update mapping
        self.mapping.insert(e, leafs);

        return Position { x: new_pos.x + moved.x, y: new_pos.y + moved.y }
    }

    pub fn remove(&mut self, e: Entity) {
        let mut leafs = match self.mapping.remove(&e) {
            None => panic!("removing nonexistant entity from CollisionWorld"),
            Some(l) => l,
        };

        self.dbvt.remove(&mut leafs.x);
        self.dbvt.remove(&mut leafs.y);
    }
}
