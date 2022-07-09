#![allow(clippy::type_complexity)]
// this imports most common types used
// in a game made in the Bevy engine
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// main() is the entry point for the program
fn main() {
    // starts building the app
    let mut app = App::build();
    app.insert_resource(WindowDescriptor {
            title: "Neon Daze".to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            resizable: false,
            ..Default::default()
        })
        // these are all the default things the engine enables
        // like sprite rendering, transform managment
        // and much much more
        .add_plugins(DefaultPlugins)
        // this adds a function that runs when
        // the app first starts
        .add_startup_system(setup.system())
        // addding the coodown system
        .add_plugin(cooldown::CooldownPlugin)
        // adding game state controller
        .add_plugin(state::StatePlugin)
        // adding the physics system
        .add_plugin(
            RapierPhysicsPlugin::<NoUserData>::default(),
        )
        // disabling gravity (as it comes enabled by default)
        .add_startup_system(disable_gravity.system())
        // adding the asset loader
        .add_plugin(asset::AssetPlugin)
        // adding the menus
        .add_plugin(menus::MenuPlugin)
        // this adds the build function of the PlayerPlugin
        // to the setup routines of the app
        // adds grid to game
        .add_plugin(grid::GridPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(enemies::EnemyPlugin)
        .add_plugin(item::ItemPlugin)
        .add_plugin(just_spawned::JustSpawnedPlugin);
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    // runs the app
    app.run();
}

// disables gravity by setting
// it to zero
fn disable_gravity(
    mut rapier_cofig: ResMut<RapierConfiguration>,
) {
    rapier_cofig.gravity = Vec2::ZERO.into();
}

// this function gets run during startup and its parameters
// are passed in from the engine
// Commands: allows spawning and despawning of entities
fn setup(mut commands: Commands) {
    // renders of sprites and such to the screen
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    // renders of UI elements over the game
    commands.spawn_bundle(UiCameraBundle::default());
}

// marker for the game's main camera
// so that it can be queried to
// convert screen positions
// to world coords
pub struct MainCamera;

// declares player as a submodule of main so main
// can access it's public members
mod player;

mod menus;

// this module is declared as public
// because many other files will need
// to access it to determine whether
// they should run or not
pub mod state;

pub mod asset;

pub mod grid;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 800.0;

pub mod phys;

pub mod cooldown;

pub mod item;

pub mod enemies;

pub mod just_spawned;
