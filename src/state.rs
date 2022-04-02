use bevy::{
    ecs::{
        component::Component,
        schedule::ParallelSystemDescriptor,
    },
    prelude::{DespawnRecursiveExt, *},
};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_state(GameState::Loading)
            .add_event::<GameEvent>()
            .add_system(
                GameEvent::event_state_control.system(),
            )
            .add_system(GameState::level_restart.system());
    }
}

// these can all be auto-implemented by Rust
// and are needed in order to control the state
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    Pause,
    LevelCountdown,
    InLevel,
    LoadingLevel,
    Loading,
    LevelRestart,
    ItemMenu,
}
// due to this type of pattern being so common Bevy
// already has internal systems to manage state
// so there is no need for any more state logic
// as it is already pre-done by the engine

impl GameState {
    // despawns everything of type T after a given
    // gamestate is exited
    pub fn despawn<T: Component>(
        after: GameState,
    ) -> ParallelSystemDescriptor {
        (|mut commmands: Commands,
          query: Query<Entity, With<T>>| {
            // iterates over every entity with that
            // component and despawns them
            for entity in query.iter() {
                commmands.entity(entity).despawn_recursive()
            }
        })
        .system()
        // making this system only run on
        // exiting the given state
        .with_run_criteria(
            State::<GameState>::on_exit(after),
        )
    }

    // when the LevelRestart state is set just re-enter
    // the InLevel state to restart the level
    pub fn level_restart(
        mut app_state: ResMut<State<GameState>>,
    ) {
        if matches!(
            app_state.current(),
            GameState::LevelRestart
        ) {
            app_state
                .overwrite_set(GameState::InLevel)
                .unwrap();
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum GameEvent {
    LevelClear,
    PlayerHit,
    GameOver,
}

impl GameEvent {
    // changes the game state based on game events
    pub fn event_state_control(
        mut events: EventReader<GameEvent>,
        mut app_state: ResMut<State<GameState>>,
    ) {
        // collecting the events
        let events = events.iter().collect::<Vec<_>>();
        // if the game is over then return to the main menu
        if events.contains(&&GameEvent::GameOver) {
            app_state
                .overwrite_set(GameState::MainMenu)
                .unwrap()
        } else if events.contains(&&GameEvent::PlayerHit) {
            // if the player is hit restart the level
            app_state
                .overwrite_set(GameState::LevelRestart)
                .unwrap()
        } else if events.contains(&&GameEvent::LevelClear) {
            // if the level is clear load the next level
            app_state
                .overwrite_set(GameState::ItemMenu)
                .ok();
        }
    }
}
