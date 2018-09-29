use ecs::{DataHelper, Entity};

use crate::{components::LevelComponents, systems::LevelServices};
use crate::systems::interaction_system;

pub trait EventReceiver<T> {
    fn receive_event(&mut self, event: T);
}

#[derive(Copy, Clone, Debug)]
pub struct CollisionStarted {
    pub collider: Entity,
    pub collided: Entity,
}

impl EventReceiver<CollisionStarted> for DataHelper<LevelComponents, LevelServices> {
    fn receive_event(&mut self, event: CollisionStarted) {
        interaction_system::on_collision_started(self, &event);
    }
}
