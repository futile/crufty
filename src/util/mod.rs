pub trait State<T> {
    fn run(self: Box<Self>) -> T;
}

pub fn run_state_machine<T, F>(initial: T, transition_func: F)
    where F: Fn(T) -> Option<Box<State<T>>>
{
    let mut transition = initial;

    while let Some(s) = transition_func(transition) {
        transition = s.run();
    }
}

#[cfg(test)]
mod test {
    use super::{State, run_state_machine};

    enum Transition {
        First,
        Last,
        Over
    }

    struct FirstState;
    struct LastState;

    impl State<Transition> for FirstState {
        fn run(self: Box<Self>) -> Transition {
            Transition::Last
        }
    }

    impl State<Transition> for LastState {
        fn run(self: Box<Self>) -> Transition {
            Transition::Over
        }
    }

    #[test]
    fn simple_state_machine() {
        run_state_machine(Transition::First, |t: Transition| {
            match t {
                Transition::First => Some(Box::new(FirstState)),
                Transition::Last => Some(Box::new(LastState)),
                Transition::Over => None
            }
        })
    }
}
