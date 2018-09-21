use typemap::{TypeMap, Key};
use std::rc::{Rc, Weak};

pub trait PipelinedEvent {
    type NextEvent: PipelinedEvent;

    fn into_next(self) -> Option<Self::NextEvent>;
}

impl PipelinedEvent for ! {
    type NextEvent = !;

    fn into_next(self) -> Option<Self::NextEvent> {
        None
    }
}

pub trait EventSubscriber<T> {
    fn handle_event(&self, event: &mut T);
}

pub struct EventPipeline {
    event_subscribers: TypeMap,
}

enum RcOrWeak<T: ?Sized> {
    Rc(Rc<T>),
    Weak(Weak<T>),
}

impl<T: ?Sized> RcOrWeak<T> {
    fn try_rc(&self) -> Option<Rc<T>> {
        match *self {
            RcOrWeak::Rc(ref rc) => Some(rc.clone()),
            RcOrWeak::Weak(ref weak) => weak.upgrade(),
        }
    }
}

struct EventWrapper<T>(::std::marker::PhantomData<*const T>);

impl<T: 'static> Key for EventWrapper<T> {
    type Value = Vec<RcOrWeak<dyn EventSubscriber<T>>>;
}

impl EventPipeline {
    pub fn new() -> EventPipeline {
        EventPipeline {
            event_subscribers: TypeMap::new(),
        }
    }

    pub fn fire_event<T: PipelinedEvent + 'static>(&mut self, mut event: T) {
        {
            let subscribers = self.event_subscribers.entry::<EventWrapper<T>>().or_insert_with(Vec::new);

            subscribers.drain_filter(|subscriber| {
                let subscriber = match subscriber.try_rc() {
                    Some(rc) => rc,
                    None => return true,
                };

                subscriber.handle_event(&mut event);

                return false;
            });
        }

        if let Some(next_event) = event.into_next() {
            self.fire_event(next_event);
        }
    }

    pub fn add_subscriber<T: 'static>(&mut self, subscriber: Rc<dyn EventSubscriber<T>>) {
        let subscribers = self.event_subscribers.entry::<EventWrapper<T>>().or_insert_with(Vec::new);
        subscribers.push(RcOrWeak::Rc(subscriber));
    }

    pub fn add_weak_subscriber<T: 'static>(&mut self, subscriber: Weak<dyn EventSubscriber<T>>) {
        let subscribers = self.event_subscribers.entry::<EventWrapper<T>>().or_insert_with(Vec::new);
        subscribers.push(RcOrWeak::Weak(subscriber));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::RefCell;

    struct SimpleEvent;

    impl PipelinedEvent for SimpleEvent {
        type NextEvent = !;

        fn into_next(self) -> Option<!> {
            None
        }
    }

    impl<T> EventSubscriber<T> for RefCell<u32> {
        fn handle_event(&self, _: &mut T) {
            *self.borrow_mut() += 1;
        }
    }

    #[test]
    fn fire_empty() {
        let mut pipeline = EventPipeline::new();
        pipeline.fire_event(SimpleEvent);
    }

    #[test]
    fn single_handler() {
        let counter = Rc::new(RefCell::new(0u32));

        let mut pipeline = EventPipeline::new();
        pipeline.add_subscriber::<SimpleEvent>(counter.clone());

        pipeline.fire_event(SimpleEvent);
        assert_eq!(*counter.borrow(), 1);

        pipeline.fire_event(SimpleEvent);
        assert_eq!(*counter.borrow(), 2);
    }

    #[test]
    fn multiple_handlers() {
        let counter = Rc::new(RefCell::new(0u32));

        let mut pipeline = EventPipeline::new();
        pipeline.add_subscriber::<SimpleEvent>(counter.clone());
        pipeline.add_subscriber::<SimpleEvent>(counter.clone());

        pipeline.fire_event(SimpleEvent);
        assert_eq!(*counter.borrow(), 2);

        pipeline.fire_event(SimpleEvent);
        assert_eq!(*counter.borrow(), 4);
    }

    #[test]
    fn weak_handler() {
        let counter = Rc::new(RefCell::new(0u32));

        let mut pipeline = EventPipeline::new();
        let weak = Rc::downgrade(&counter);
        pipeline.add_weak_subscriber::<SimpleEvent>(weak);

        pipeline.fire_event(SimpleEvent);
        assert_eq!(*counter.borrow(), 1);

        // Drop the single strong count by unwrapping the Rc.
        let counter = Rc::try_unwrap(counter).unwrap();

        // This should *not* trigger the handler again, since the only strong reference was dropped
        pipeline.fire_event(SimpleEvent);
        assert_eq!(*counter.borrow(), 1);
    }

    struct CountedEvent(u32);

    impl PipelinedEvent for CountedEvent {
        type NextEvent = CountedEvent;

        fn into_next(self) -> Option<CountedEvent> {
            match self.0 {
                0 => None,
                x => Some(CountedEvent(x-1)),
            }
        }
    }

    #[test]
    fn counted_event() {
        const COUNT: u32 = 5;
        let counter = Rc::new(RefCell::new(0u32));

        let mut pipeline = EventPipeline::new();
        pipeline.add_subscriber::<CountedEvent>(counter.clone());

        pipeline.fire_event(CountedEvent(COUNT));
        assert_eq!(*counter.borrow(), COUNT+1);
    }
}
