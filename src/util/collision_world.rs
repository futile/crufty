use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use ecs::Entity;

use nc::partitioning::{DBVT, DBVTLeaf};
use nc::bounding_volume::AABB2;
use na::Pnt2;

use components::Collision;

type CollisionTreeLeaf = Rc<RefCell<DBVTLeaf<Pnt2<f32>, Entity, AABB2<f32>>>>;

struct CollisionTreeLeafs {
    x: CollisionTreeLeaf,
    y: CollisionTreeLeaf,
}

pub struct CollisionWorld {
    dbvt: DBVT<Pnt2<f32>, Entity, AABB2<f32>>,
    mapping: HashMap<Entity, CollisionTreeLeafs>
}

impl CollisionWorld {
    pub fn new() -> CollisionWorld {
        CollisionWorld {
            dbvt: DBVT::new(),
            mapping: HashMap::new(),
        }
    }

    pub fn add(&mut self, e: Entity, coll: &Collision) {
        // TODO dbvt.insert(), mapping.insert()
        // let (leaf_x, leaf_y) = {
        //     let coll: &Collision = &data.collision[*e];
        //     let pos:  &Position  = &data.position[*e];

        //     (services.collision_tree.insert_new(***e, coll.aabb_x(Vec2::new(pos.x, pos.y))),
        //      services.collision_tree.insert_new(***e, coll.aabb_y(Vec2::new(pos.x, pos.y))))
        // };
    }

    pub fn remove(&mut self, e: Entity) {
        // TODO dbvt.remove(), mapping.remove()
    }
}
