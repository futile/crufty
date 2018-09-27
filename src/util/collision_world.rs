use std::collections::HashMap;

use std::cell::RefCell;

use ecs::Entity;

use crate::nc::bounding_volume::BoundingVolume;
use crate::nc::bounding_volume::AABB;
use crate::nc::math::{Point, Vector};
use crate::nc::partitioning::BoundingVolumeInterferencesCollector;
use crate::nc::partitioning::{DBVTLeaf, DBVTLeafId, DBVT};
use crate::nc::query::Ray;
use crate::nc::query::RayInterferencesCollector;

use crate::components::{self, CollisionType, Position};

use ordered_float::NotNan;

type CollisionTreeLeafId = DBVTLeafId;
type CollisionTreeLeaf = DBVTLeaf<f32, Entity, AABB<f32>>;

struct CollisionTreeLeafs {
    x: CollisionTreeLeafId,
    y: CollisionTreeLeafId,
    coll_type: CollisionType,
}

pub struct CollisionWorld {
    dbvt_x: DBVT<f32, Entity, AABB<f32>>,
    dbvt_y: DBVT<f32, Entity, AABB<f32>>,
    mapping: HashMap<Entity, CollisionTreeLeafs>,
    on_ground_cache: RefCell<HashMap<Entity, bool>>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Axis {
    X,
    Y,
}

fn find_depth(
    dyn_ent: &AABB<f32>,
    dyn_last: &Point<f32>,
    stat: &AABB<f32>,
    axis: Axis,
) -> Option<f32> {
    use self::Axis::{X, Y};

    if !dyn_ent.intersects(stat) {
        return None;
    }

    let min_dist = match axis {
        X => dyn_ent.half_extents().x + stat.half_extents().x,
        Y => dyn_ent.half_extents().y + stat.half_extents().y,
    };

    let dist = match axis {
        X => dyn_ent.center().x - stat.center().x,
        Y => dyn_ent.center().y - stat.center().y,
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

pub struct CollisionResult {
    depth: f32,
    other: Entity,
    other_coll_type: CollisionType,
}

impl CollisionResult {
    fn new(depth: f32, other: Entity, other_coll_type: CollisionType) -> CollisionResult {
        CollisionResult {
            depth: depth,
            other: other,
            other_coll_type: other_coll_type,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Collision {
    pub collider: Entity,
    pub collided: Entity,
}

impl CollisionWorld {
    pub fn new() -> CollisionWorld {
        CollisionWorld {
            dbvt_x: DBVT::new(),
            dbvt_y: DBVT::new(),
            mapping: HashMap::new(),
            on_ground_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn add(&mut self, e: Entity, coll: &components::Collision, pos: &Position) {
        // if it already existed, just remove it
        self.remove(e);

        let x_leaf = DBVTLeaf::new(coll.aabb_x(Vector::new(pos.x, pos.y)), e);
        let y_leaf = DBVTLeaf::new(coll.aabb_y(Vector::new(pos.x, pos.y)), e);

        // then add leafs for both trees
        self.mapping.insert(
            e,
            CollisionTreeLeafs {
                x: self.dbvt_x.insert(x_leaf),
                y: self.dbvt_y.insert(y_leaf),
                coll_type: coll.collision_type(),
            },
        );
    }

    // move an entity along one exis, return collision depth if a collision occured
    fn move_axis(
        &mut self,
        leafs: (&mut CollisionTreeLeaf, &mut CollisionTreeLeaf),
        coll: &components::Collision,
        new_pos: &Position,
        _last_pos: Option<&Position>, // currently unused, see comment below
        axis: Axis,
    ) -> Option<CollisionResult> {
        let aabb = match axis {
            Axis::X => coll.aabb_x(new_pos.as_vec()),
            Axis::Y => coll.aabb_y(new_pos.as_vec()),
            // this caused problems, but was taken from the cavestory tutorial.
            // keeping it to not forget about it!
            // Axis::X => coll.aabb_x(new_pos.as_vec()).merged(&coll.aabb_x(last_pos.as_vec())),
            // Axis::Y => coll.aabb_y(new_pos.as_vec()).merged(&coll.aabb_y(last_pos.as_vec())),
        };

        let leaf = match axis {
            Axis::X => leafs.0,
            Axis::Y => leafs.1,
            // Axis::X => self.dbvt_x[leafs.x],
            // Axis::Y => self.dbvt_y[leafs.y],
        };

        // find closest colliding entity
        let mut colls: Vec<Entity> = Vec::new();
        let closest: Option<&Entity> = match axis {
            Axis::X => {
                self.dbvt_x
                    .visit(&mut BoundingVolumeInterferencesCollector::new(
                        &aabb, &mut colls,
                    ));

                let center_x = leaf.center;

                colls.iter().min_by_key(|other| {
                    let other_leafs = self.mapping.get(other).unwrap();
                    NotNan::new((self.dbvt_x[other_leafs.x].center.x - center_x.x).abs()).unwrap()
                })
            }
            Axis::Y => {
                self.dbvt_y
                    .visit(&mut BoundingVolumeInterferencesCollector::new(
                        &aabb, &mut colls,
                    ));

                let center_y = leaf.center;

                colls.iter().min_by_key(|other| {
                    let other_leafs = self.mapping.get(other).unwrap();
                    NotNan::new((self.dbvt_y[other_leafs.y].center.y - center_y.y).abs()).unwrap()
                })
            }
        };

        // if one was found, test again, this time only in one direction (dbvt tests in both)
        if let Some(other) = closest {
            let other_leafs = self.mapping.get(other).unwrap();

            let other_leaf = match axis {
                Axis::X => &self.dbvt_x[other_leafs.x],
                Axis::Y => &self.dbvt_y[other_leafs.y],
            };

            let depth = find_depth(&aabb, &leaf.center, &other_leaf.bounding_volume, axis);

            if let Some(depth) = depth {
                return Some(CollisionResult::new(depth, *other, other_leafs.coll_type));
            }
        };

        None
    }

    pub fn move_entity<E: Extend<Collision>>(
        &mut self,
        e: Entity,
        coll: &components::Collision,
        new_pos: &Position,
        last_pos: Option<&Position>,
        collision_collector: &mut E,
    ) -> Position {
        // 1. remove both leafs
        let mut leafs: CollisionTreeLeafs = self.mapping.remove(&e).unwrap();

        let mut leaf_x = self.dbvt_x.remove(leafs.x);
        let mut leaf_y = self.dbvt_y.remove(leafs.y);

        let mut updated_pos = *new_pos;

        // 2. call move_axis for both axes, X first
        if let Some(col_result) = self.move_axis(
            (&mut leaf_x, &mut leaf_y),
            coll,
            &updated_pos,
            last_pos,
            Axis::X,
        ) {
            if leafs.coll_type == CollisionType::Solid
                && col_result.other_coll_type == CollisionType::Solid
            {
                updated_pos.x += col_result.depth;
            } else {
                collision_collector.extend(std::iter::once(Collision {
                    collider: e,
                    collided: col_result.other,
                }));
            }
        }

        if let Some(col_result) = self.move_axis(
            (&mut leaf_x, &mut leaf_y),
            coll,
            &updated_pos,
            last_pos,
            Axis::Y,
        ) {
            if leafs.coll_type == CollisionType::Solid
                && col_result.other_coll_type == CollisionType::Solid
            {
                updated_pos.y += col_result.depth;
            } else {
                collision_collector.extend(std::iter::once(Collision {
                    collider: e,
                    collided: col_result.other,
                }));
            }
        }

        let new_center = updated_pos.as_pnt();
        leaf_x = DBVTLeaf::new(coll.aabb_x(new_center.coords), e);

        let new_center = updated_pos.as_pnt(); // + *coll.off_y();
        leaf_y = DBVTLeaf::new(coll.aabb_y(new_center.coords), e);

        // 3. re-insert into trees
        leafs.x = self.dbvt_x.insert(leaf_x);
        leafs.y = self.dbvt_y.insert(leaf_y);

        // 4. update mapping
        self.mapping.insert(e, leafs);

        // clear on_ground cache
        self.on_ground_cache.borrow_mut().clear();

        // return new position after collisions have been resolved
        updated_pos
    }

    pub fn on_ground(&self, e: Entity) -> bool {
        if let Some(on_ground) = self.on_ground_cache.borrow().get(&e) {
            return *on_ground;
        }

        let leafs: &CollisionTreeLeafs = self.mapping.get(&e).unwrap();
        let leaf_y = &self.dbvt_y[leafs.y];

        let mut colls = Vec::new();
        self.dbvt_y.visit(&mut RayInterferencesCollector::new(
            &Ray::new(leaf_y.center, Vector::new(0.0, -1.0)),
            &mut colls,
        ));

        let bot_y = leaf_y.bounding_volume.mins().y;

        const ON_GROUND_THRESHOLD: f32 = 0.000015; // chosen through experiments

        let on_ground = colls
            .iter()
            .filter(|other| e != **other) // no self-collisions
            .filter_map(|other| {
                let other_leafs = self.mapping.get(other).unwrap();
                if other_leafs.coll_type != CollisionType::Solid {
                    return None;
                }
                let other_leaf_y = &self.dbvt_y[other_leafs.y];
                let other_top_y = other_leaf_y.bounding_volume.maxs().y;
                let dist = (other_top_y - bot_y).abs(); // maybe remove the abs(), but shouldn't matter too much
                Some(dist)
            }).any(|dist| dist < ON_GROUND_THRESHOLD);

        self.on_ground_cache.borrow_mut().insert(e, on_ground);

        on_ground
    }

    pub fn remove(&mut self, e: Entity) {
        let leafs = match self.mapping.remove(&e) {
            Some(l) => l,
            None => return,
        };

        self.dbvt_x.remove(leafs.x);
        self.dbvt_y.remove(leafs.y);
    }
}
