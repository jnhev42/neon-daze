use std::time::Duration;

use crate::state;
use bevy::prelude::*;

pub struct CooldownPlugin;

impl Plugin for CooldownPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(
            // ticks the cooldowns
            Cooldown::tick.system().with_run_criteria(
                State::<state::GameState>::on_update(
                    state::GameState::InLevel,
                ),
            ),
        );
    }
}

// this is a wrapper for bevy's builtin
// timer component to make code that handles
// time more convinent
// also allows there to be no cooldown to
// totally disable
#[derive(Debug)]
pub struct Cooldown {
    timer: Option<Timer>,
}

impl Cooldown {
    // private helper function for creating a timer
    // annotated as inline to speed up at runtime
    #[inline]
    fn create_timer(secs: f32) -> Timer {
        Timer::from_seconds(secs, false)
    }

    // returns whether a timer is over
    // returns false when no timer is created
    pub fn is_over(&self) -> bool {
        match self.timer {
            Some(ref timer) => timer.finished(),
            None => false,
        }
    }

    // sets the cooldown to a given value
    pub fn set(&mut self, secs: f32) {
        self.timer = Some(Self::create_timer(secs))
    }

    // restarts the cooldown
    pub fn reset(&mut self) {
        if let Some(ref mut timer) = self.timer {
            timer.reset()
        }
    }

    // creates a new (possibly null) cooldown
    // that lasts the specified number of seconds
    pub fn new(secs: Option<f32>) -> Self {
        let timer = secs.map(Self::create_timer);
        Self { timer }
    }

    // manually sets the time elapsed on the timer
    pub fn set_elapsed(&mut self, secs: f32) {
        if let Some(ref mut timer) = self.timer {
            timer.set_elapsed(Duration::from_secs_f32(secs))
        };
    }

    // updates each cooldown's internal
    // timer with the engine's meausured time
    // delta
    pub fn tick(
        time: Res<Time>,
        mut query: Query<&mut Cooldown>,
    ) {
        // calculating the time elapsed since the last frame
        let delta = time.delta();
        // advancing all the cooldown timers by that amount
        for mut cooldown in query.iter_mut() {
            if let Some(ref mut timer) = cooldown.timer {
                timer.tick(delta);
            }
        }
    }
    // (it is possible that this system
    // ignores the game being paused
    // and ticks cooldown anyway but
    // thats a problem for future me)
}
