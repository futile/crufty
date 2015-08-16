use ecs::{ System, DataHelper, EntityIter, EntityData, Entity };
use ecs::system::InteractProcess;

use super::LevelServices;

use components::{LevelComponents, Position, Collision, CollisionType};

use na::{Pnt2, Vec2};
use nc::bounding_volume::{BoundingVolume, AABB2};

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

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Axis {
    X,
    Y,
}

fn find_depth(dyn: &AABB2<f32>, dyn_last: Pnt2<f32>, stat: &AABB2<f32>, axis: Axis) -> Option<f32> {
    use self::Axis::{X,Y};

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

impl InteractProcess for CollisionSystem {
    fn process(&mut self, dynamic_entities: EntityIter<LevelComponents>, static_entities_iter: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, LevelServices>) {
        let static_entities: Vec<_>= static_entities_iter.collect();

        for e1 in dynamic_entities {
            let mut p1 = data.position[e1];
            let v1 = data.velocity[e1];
            let c1 = data.collision[e1].clone();

            for e2 in &static_entities {
                if **e1 == ***e2 {
                    continue;
                }

                {
                    let p2 = &data.position[*e2];
                    let c2 = &data.collision[*e2];
                    let v2 = data.velocity.get(e2);

                    let other_pos = v2.as_ref().map_or(Vec2::new(p2.x, p2.y), |vel| Vec2::new(vel.last_pos.x, vel.last_pos.y));
                    let aabb2_x = c2.aabb_x(Vec2::new(p2.x, p2.y)).merged(&c2.aabb_x(other_pos));

                    if c1.collision_type() != CollisionType::Solid || c2.collision_type() != CollisionType::Solid {
                        continue;
                    }

                    let aabb1_x = c1.aabb_x(Vec2::new(p1.x, p1.y)).merged(&c1.aabb_x(Vec2::new(v1.last_pos.x, v1.last_pos.y)));

                    let aabb1_y = c1.aabb_y(Vec2::new(p1.x, p1.y)).merged(&c1.aabb_y(Vec2::new(v1.last_pos.x, v1.last_pos.y)));
                    let aabb2_y = c2.aabb_y(Vec2::new(p2.x, p2.y)).merged(&c2.aabb_y(other_pos));

                    if let Some(depth_x) = find_depth(&aabb1_x, Pnt2::new(v1.last_pos.x, v1.last_pos.y), &aabb2_x, Axis::X) {
                        p1.x += depth_x;
                    } else if let Some(depth_y) = find_depth(&aabb1_y, Pnt2::new(v1.last_pos.x, v1.last_pos.y), &aabb2_y, Axis::Y) {
                        p1.y += depth_y;
                    }
                }

                data.position[e1] = p1;
            }
        }
    }
}
