use crate::state;
use bevy::prelude::*;

// this stores information about the current
// difficulty for the game
pub struct Difficulty {
    level_num: u32,
}

// making the starting level one
impl Default for Difficulty {
    fn default() -> Self {
        Self { level_num: 1 }
    }
}

impl Difficulty {
    // returns a copy of the level
    // num to prevent anything but
    // Difficulty's internal systems
    // from modifying it
    pub fn level(&self) -> u32 {
        self.level_num
    }

    // returns the points generate can spend on
    // spawning enemies in a level
    pub fn points(&self) -> f32 {
        let num = self.level_num as f32;
        // gives a reasonably useful graph
        // that scales quickly at first but
        // after level 5 is basically
        // linear since the first number is so small
        // so that early on the difficulty scales faster
        // but doesn't keep scaling at that same rate
        // forever so it doesn't become unbeatable
        -1000.0 / num.sqrt().sqrt() + 15.0 * num + 1300.0
    }

    // increments the level_num whenever a level is cleared
    pub fn increment_level(
        mut difficulty: ResMut<Difficulty>,
        mut game_events: EventReader<state::GameEvent>,
    ) {
        if game_events.iter().any(|ev| {
            matches!(ev, state::GameEvent::LevelClear)
        }) {
            difficulty.level_num += 1;
        }
    }

    // resets the level_num when the game is over
    pub fn reset(
        mut difficulty: ResMut<Difficulty>,
        mut game_events: EventReader<state::GameEvent>,
    ) {
        if game_events.iter().any(|ev| {
            matches!(ev, state::GameEvent::GameOver)
        }) {
            *difficulty = Default::default();
        }
    }
}
