use ecs::system::EntityProcess;
use ecs::{DataHelper, EntityIter, System};

use crate::application::InputIntent;
use crate::game::events::{CollisionEnded, CollisionStarted, EventReceiver, InteractionDone};
use crate::game::{EntityOps, Interaction};
use crate::{components::LevelComponents, components::Position, systems::LevelServices};

pub struct InteractionSystem;

impl System for InteractionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for InteractionSystem {
    fn process(
        &mut self,
        entities: EntityIter<'_, LevelComponents>,
        data: &mut DataHelper<LevelComponents, LevelServices>,
    ) {
        for e in entities {
            let wants_interaction = data.intents[e].remove(&InputIntent::Interact);

            if !wants_interaction {
                continue;
            }

            let interactor = data.interactor[e];
            let pos = data.position[e];
            let mut interaction_target = None;
            for other in data.collision_shape[e].ongoing_collisions.others.clone() {
                match data.with_entity_data(&other, |en, comps| {
                    comps
                        .interaction_possibility
                        .get(&en)
                        .map(|poss| interactor.can_interact(&poss.interaction))
                }) {
                    Some(Some(true)) => (),
                    _ => continue,
                };

                let other_pos = match data.with_entity_data(&other, |en, comps| comps.position[en])
                {
                    Some(p) => p,
                    _ => continue,
                };

                let distance =
                    ((other_pos.x - pos.x).powf(2.0) + (other_pos.y - pos.y).powf(2.0)).sqrt();

                if let Some((_, curr_dist)) = interaction_target {
                    if distance < curr_dist {
                        interaction_target = Some((other, distance));
                    }
                } else {
                    interaction_target = Some((other, distance));
                }
            }

            let interaction_target = match interaction_target {
                Some((e, _)) => e,
                _ => continue,
            };

            let interaction_possibility = data
                .with_entity_data(&interaction_target, |en, comps| {
                    comps.interaction_possibility[en]
                })
                .unwrap();

            match interaction_possibility.interaction {
                Interaction::WarpInRoom { x, y } => {
                    data.move_entity(e.into(), Position { x, y }, true);
                    data.receive_event(InteractionDone {
                        interactor: **e,
                        interacted: interaction_target,
                        interaction: interaction_possibility.interaction,
                    })
                }
            };
        }
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
            .map(|i| i.can_interact(&interaction))
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
            .map(|i| i.can_interact(&interaction))
    }) {
        Some(Some(i)) => i,
        _ => return,
    };

    if can_interact {
        println!("can interact end");
    }
}
