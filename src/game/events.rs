use ecs::{DataHelper, Entity};

use crate::{components::LevelComponents, systems::LevelServices};

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
        let interaction = match self.with_entity_data(&event.collided, |en, comps| {
            comps
                .interaction_possibility
                .borrow(&en)
                .map(|ip| ip.interaction)
        }) {
            Some(Some(i)) => i,
            _ => return,
        };

        let can_interact = match self.with_entity_data(&event.collider, |en, comps| {
            comps
                .interactor
                .borrow(&en)
                .map(|i| i.can_interact(interaction))
        }) {
            Some(Some(i)) => i,
            _ => return,
        };

        if can_interact {
            println!("can interact!!!!!!!!");
        }
    }
}
