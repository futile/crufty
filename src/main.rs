mod util;
mod application;

use util::{run_state_machine};
use application::{AppTransition, StartupState, GameState, ShutdownState};

#[allow(dead_code)]
fn main() {
    run_state_machine(AppTransition::Startup, |t| {
        match t {
            AppTransition::Startup => Some(Box::new(StartupState)),
            AppTransition::StartGame => Some(Box::new(GameState)),
            AppTransition::Shutdown => Some(Box::new(ShutdownState)),
            AppTransition::TerminateApplication => None
        }
    })
}
