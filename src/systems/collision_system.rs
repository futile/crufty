use ecs::{ System, DataHelper, EntityIter, EntityData, Entity };
use ecs::system::InteractProcess;

use super::LevelServices;

use components::{LevelComponents, Position, Collision, CollisionType};

use na::{Vec2};
use nc::bounding_volume::{BoundingVolume};

pub struct CollisionSystem;

impl CollisionSystem {
    pub fn new() -> CollisionSystem {
        CollisionSystem
    }
}

impl System for CollisionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl InteractProcess for CollisionSystem {
    fn process(&mut self, dynamic_entities: EntityIter<LevelComponents>, static_entities_iter: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, LevelServices>) {
        let static_entities: Vec<_>= static_entities_iter.collect();

        for e1 in dynamic_entities {
            let mut p1 = data.position[e1];
            let v1 = data.velocity[e1];
            let c1 = data.collision[e1].clone();

            let aabb1_x = c1.aabb_x(Vec2::new(p1.x, p1.y)).merged(&c1.aabb_x(Vec2::new(v1.last_pos.x, v1.last_pos.y)));

            // only create after x resolved!
            // let aabb1_y = c1.aabb_y(Vec2::new(p1.x, p1.y)).merged(&c1.aabb_y(Vec2::new(v1.last_pos.x, v1.last_pos.y)));

            for e2 in &static_entities {
                if **e1 == ***e2 {
                    continue;
                }

                let p2 = &data.position[*e2];
                let c2 = &data.collision[*e2];
                let v2 = data.velocity.get(e2);

                let other_pos = v2.as_ref().map_or(Vec2::new(p2.x, p2.y), |vel| Vec2::new(vel.last_pos.x, vel.last_pos.y));

                let aabb2_x = c2.aabb_x(Vec2::new(p2.x, p2.y)).merged(&c2.aabb_x(other_pos));

                // only create after x resolved!
                // let aabb2_y = c2.aabb_y(Vec2::new(p2.x, p2.y)).merged(&c2.aabb_y(other_pos));

                // TODO fire event on collision

                if c1.collision_type() != CollisionType::Solid || c2.collision_type() != CollisionType::Solid {
                    continue;
                }

                // 1. detect if collision
                let depth_x = if aabb1_x.intersects(&aabb2_x) {
                    Some(if v1.last_pos.x <= p1.x {
                        aabb1_x.maxs().x - aabb2_x.mins().x
                    } else {
                        aabb1_x.mins().x - aabb2_x.maxs().x
                    })
                } else {
                    None
                };

                // 2. if both SOLID, resolve collision
                if let Some(dx) = depth_x {
                    p1.x -= dx;
                }
            }

            data.position[e1] = p1;
        }
    }
}
