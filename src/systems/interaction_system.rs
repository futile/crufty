use ecs::system::EntityProcess;
use ecs::{DataHelper, EntityIter, System};

use crate::game::events::{CollisionStarted, CollisionEnded};
use crate::{components::LevelComponents, systems::LevelServices};

//use game::Interaction;

pub struct InteractionSystem;

impl System for InteractionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for InteractionSystem {
    fn process(
        &mut self,
        _entities: EntityIter<'_, LevelComponents>,
        _data: &mut DataHelper<LevelComponents, LevelServices>,
    ) {
    }
}

pub fn on_collision_started(
    data: &mut DataHelper<LevelComponents, LevelServices>,
    event: &CollisionStarted,
) {
    let interaction = match data.with_entity_data(&event.collided, |en, comps| {
        comps
            .interaction_possibility
            .borrow(&en)
            .map(|ip| ip.interaction)
    }) {
        Some(Some(i)) => i,
        _ => return,
    };

    let can_interact = match data.with_entity_data(&event.collider, |en, comps| {
        comps
            .interactor
            .borrow(&en)
            .map(|i| i.can_interact(interaction))
    }) {
        Some(Some(i)) => i,
        _ => return,
    };

    if can_interact {
        println!("can interact start");
    }
}

pub fn on_collision_ended(
    data: &mut DataHelper<LevelComponents, LevelServices>,
    event: &CollisionEnded,
) {
    let interaction = match data.with_entity_data(&event.collided, |en, comps| {
        comps
            .interaction_possibility
            .borrow(&en)
            .map(|ip| ip.interaction)
    }) {
        Some(Some(i)) => i,
        _ => return,
    };

    let can_interact = match data.with_entity_data(&event.collider, |en, comps| {
        comps
            .interactor
            .borrow(&en)
            .map(|i| i.can_interact(interaction))
    }) {
        Some(Some(i)) => i,
        _ => return,
    };

    if can_interact {
        println!("can interact end");
    }
}
