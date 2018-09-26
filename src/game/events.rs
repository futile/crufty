// use ecs::{DataHelper, Entity, EntityData};

pub trait EventReceiver<T> {
    fn receive_event(&mut self, event: T);
}
