use crate::state;
use bevy::prelude::*;

// putting file pathes in one centralised
// place so they're easier to find if
// they need to be changed later
mod file_path {
    pub const FONT: &str = "fonts/SkyhookMono.ttf";
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // initialises the Materials struct in
            // the games resources at startup
            .init_resource::<Materials>()
            // adds the system that checks to see
            // if assets are loaded
            .add_system_set(
                SystemSet::on_update(
                    state::GameState::Loading,
                )
                .with_system(
                    Materials::check_loaded.system(),
                ),
            )
            .insert_resource(ClearColor(
                Color::hex("14080E").unwrap(),
            ));
    }
}

// stores handles for all the colors that
// are in the game for the gpu to render
// also stores font and file handles
pub struct Materials {
    pub player_body: Handle<ColorMaterial>,
    pub button_normal: Handle<ColorMaterial>,
    pub tile_empty: Handle<ColorMaterial>,
    pub tile_wall: Handle<ColorMaterial>,
    pub player_gun: Handle<ColorMaterial>,
    pub enemy: Handle<ColorMaterial>,
    pub main_font: Handle<Font>,
}

impl FromWorld for Materials {
    // called when intialising the game
    fn from_world(world: &mut World) -> Self {
        // creates and manages handles to colors for the shader
        let mut colors = world
            .get_resource_mut::<Assets<ColorMaterial>>()
            .unwrap();
        // macro to reduce boilerplate
        macro_rules! hex {
            ($hex:literal) => {
                colors.add(Color::hex($hex).unwrap().into())
            };
        }

        // creating handle to the color of the player
        let player_body = hex!("0038A8");
        // creating handle to the color of the main menu button
        let button_normal = hex!("14080e");
        // adding new tile color materials
        let tile_empty = hex!("14080E");
        let tile_wall = hex!("271c47");
        let player_gun = hex!("D70270");
        let enemy = hex!("734F96");
        // retriving the asset server to allow loading
        // of more complex assets (from the filesystem)
        let asset_server =
            world.get_resource::<AssetServer>().unwrap();
        // loading the font from it's path
        let main_font = asset_server.load(file_path::FONT);

        Self {
            player_body,
            button_normal,
            tile_empty,
            tile_wall,
            player_gun,
            main_font,
            enemy,
        }
    }
}

impl Materials {
    // checks to see if assets are loaded in the loading screen
    fn check_loaded(
        asset_server: Res<AssetServer>,
        materials: Res<Materials>,
        mut game_state: ResMut<State<state::GameState>>,
    ) {
        // list of all the assets that should be waited
        // on to load, so every field of materials
        // besides colors as they load instantaneously
        // right now we only have the font but later
        // i'm going to need to add more
        let assets = [materials.main_font.id];
        // checks to see if all the assets are loaded
        match asset_server.get_group_load_state(assets) {
            // if all of them are loaded then enter the main menu
            bevy::asset::LoadState::Loaded => {
                game_state
                    .set(state::GameState::MainMenu)
                    .unwrap();
            }
            // if any of the materials can't load
            // then the game crashes
            bevy::asset::LoadState::Failed => {
                panic!("materials couldn't load")
            }
            // doesn't do anything otherwise
            _ => {}
        }
    }
}
