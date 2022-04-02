// adding more imports because they're needed here
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
// importing state module
use crate::{asset, grid, item, player, state};
use rand::rngs::ThreadRng;

// same as PlayerPlugin
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // calls the build on main menu
        MainMenu::build(app);
        PauseMenu::build(app);
        Countdown::build(app);
        ItemMenu::build(app);
    }
}

// holds data about any UI elements
// on the main menu
#[derive(Clone, Debug)]
enum MainMenu {
    // this represents a simple button
    // with text on it that is a string
    Button { text: String },
}

impl MainMenu {
    // adds the systems that control the main menu
    fn build(app: &mut AppBuilder) {
        app.add_system_set(
            // runs when the main menu state is first entered
            SystemSet::on_enter(state::GameState::MainMenu)
                .with_system(MainMenu::setup.system()),
        )
        .add_system_set(
            // runs every frame in the main menu
            SystemSet::on_update(
                state::GameState::MainMenu,
            )
            .with_system(MainMenu::update.system()),
        )
        .add_system(state::GameState::despawn::<
            MainMenu,
        >(
            state::GameState::MainMenu
        ));
    }

    // sets up the ui
    fn setup(
        mut commands: Commands,
        // controls the loading of assets from a filesystem
        // like fonts and images
        asset_server: Res<AssetServer>,
        materials: Res<asset::Materials>,
    ) {
        // creating a new button with the text Play
        let play_button = MainMenu::Button {
            text: "Play".to_string(),
        };
        // displaying that button on screen
        play_button.spawn(
            &mut commands,
            &asset_server,
            &*materials,
        );
    }
    fn spawn(
        self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        materials: &asset::Materials,
    ) {
        match self {
            MainMenu::Button { ref text } => {
                // create a new empty entity
                let mut entity = commands.spawn();
                // make it into a button
                entity.insert_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(
                            Val::Percent(30.0),
                            Val::Percent(10.0),
                        ),
                        // center button
                        margin: Rect::all(Val::Auto),
                        // horizontally center child text
                        justify_content:
                            JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    // make the button black
                    material: materials
                        .button_normal
                        .clone(),
                    ..Default::default()
                });
                // add on buttons data for processing when clicked
                entity.insert(self.clone());
                // add a child of the button which displays
                // text that's aligned with that button
                entity.with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        // adds some text
                        text: Text::with_section(
                            // our actual text from MainMenu::Button
                            text,
                            TextStyle {
                                // setting the font
                                font: asset_server.load(
                                    "fonts/SkyhookMono.ttf",
                                ),
                                // setting its size
                                font_size: 40.0,
                                // setting its color
                                color: Color::rgb(
                                    0.9, 0.9, 0.9,
                                ),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
            }
        }
    }
    // runs every frame to handle ui interactions
    fn update(
        mut game_state: ResMut<State<state::GameState>>,
        query: Query<
            (&Interaction, &MainMenu),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        // whilst there is only one button
        // for now there will be more
        // and all of them must work
        for (interaction, elem) in query.iter() {
            // pattern matching to filter out any events that aren't a button
            // with text being clicked
            if let (
                Interaction::Clicked,
                MainMenu::Button { text },
            ) = (interaction, elem)
            {
                // whilst matching against raw strings
                // is pretty error prone, it shortens
                // code significantly and means that
                // a button has to do what the text on it says
                match text.as_str() {
                    // if the clicked button was the play button
                    "Play" => {
                        // sets the GameState to InLevel, removing MainMenu
                        game_state
                            .set(state::GameState::LoadingLevel)
                            .unwrap();
                    }
                    // just crash if a button that has invalid text is clicked
                    other => panic!(
                        "Unrecognised button: {}",
                        other
                    ),
                }
            }
        }
    }
}

struct PauseMenu;

impl PauseMenu {
    pub fn build(app: &mut AppBuilder) {
        app.add_system(PauseMenu::enter_or_exit.system())
            .add_system_set(
                SystemSet::on_enter(
                    state::GameState::Pause,
                )
                .with_system(PauseMenu::spawn.system()),
            )
            .add_system(state::GameState::despawn::<
                PauseMenu,
            >(
                state::GameState::Pause
            ));
    }

    // handles entering/exiting the pause menu
    fn enter_or_exit(
        mut app_state: ResMut<State<state::GameState>>,
        mut keys: ResMut<Input<KeyCode>>,
        mut is_pause_held: Local<bool>,
        mut rapier_cofig: ResMut<RapierConfiguration>,
    ) {
        // if they've just been released updates
        // is_pause_held so that holding down
        // the keys doesn't open and close the
        // menu over and over again
        if keys.just_released(KeyCode::Escape)
            || keys.just_released(KeyCode::P)
        {
            *is_pause_held = false;
        }
        // checks if the keys have been just pressed
        if (keys.just_pressed(KeyCode::Escape)
            || keys.just_pressed(KeyCode::P))
            && !*is_pause_held
        {
            // matching against the current app state
            // and if it returns an error logging it
            if let Err(e) = match *app_state.current() {
                // if in the level, pause the game
                state::GameState::InLevel => {
                    rapier_cofig.physics_pipeline_active =
                        false;
                    app_state.push(state::GameState::Pause)
                }
                // if in the pause menu remove it from the top of the stack
                state::GameState::Pause => {
                    rapier_cofig.physics_pipeline_active =
                        true;
                    app_state.pop()
                }
                // otherwise not in a valid state to pause
                // so just skip completely
                _ => Ok(()),
            } {
                error!(
                    "Couldn't enter/exit pause menu: {}",
                    e
                );
            }
            // reseting keys for engine jank reasons
            keys.reset(KeyCode::Escape);
            keys.reset(KeyCode::P);
            *is_pause_held = true;
        }
    }

    // spawns in the pause menu indicator
    fn spawn(
        mut commands: Commands,
        materials: Res<asset::Materials>,
        difficulty: Res<grid::Difficulty>,
        items: Res<item::ItemManager>,
        lives: Res<player::Lives>,
    ) {
        let mut text = Text::with_section(
            "Paused\n",
            TextStyle {
                font: materials.main_font.clone(),
                font_size: 40.0,
                color: Color::rgb(0.9, 0.9, 0.9),
            },
            TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        );
        text.sections.push(TextSection {
            value: format!(
                "Level {}\n",
                difficulty.level()
            ),
            style: TextStyle {
                font: materials.main_font.clone(),
                font_size: 20.0,
                color: Color::rgb(0.9, 0.9, 0.9),
            },
        });
        text.sections.push(TextSection {
            value: format!("Lives: {}\n", lives.0),
            style: TextStyle {
                font: materials.main_font.clone(),
                font_size: 20.0,
                color: Color::rgb(0.9, 0.9, 0.9),
            },
        });
        text.sections.push(TextSection {
            value: format!("Items: {}\n", items.list()),
            style: TextStyle {
                font: materials.main_font.clone(),
                font_size: 20.0,
                color: Color::rgb(0.9, 0.9, 0.9),
            },
        });
        commands
            .spawn_bundle(TextBundle {
                text,
                ..Default::default()
            })
            .insert(PauseMenu);
    }
}

// adds an onscreen countdown to the
// level starting each time the player
// spawns in to slow down the pace of
// the game a little so the player
// has time to asses the situation
pub struct Countdown {
    timer: Timer,
    start: f32,
}

impl Countdown {
    // creates a new countdown
    // from represents the number
    // it counts down from
    pub fn new(from: f32) -> Self {
        Self {
            timer: Timer::from_seconds(
                // since actual secounds
                // would be too long and
                // slow the game down too
                // much instead count in
                // third second intervals
                from / 3.0,
                false,
            ),
            // due to the way this displays
            // i have to add 0.5 so that it
            // never displays zero which would
            // look off
            start: from + 0.5,
        }
    }
}

// for now the default countodown is from 3
impl Default for Countdown {
    fn default() -> Self {
        Self::new(3.0)
    }
}

impl Countdown {
    // adding the cooldowns systems to the game
    pub fn build(app: &mut AppBuilder) {
        app.add_system_set(
            // when entering the level spawn the countdown
            // entity to display the countdown
            SystemSet::on_enter(state::GameState::InLevel)
                .with_system(Countdown::spawn.system()),
        )
        .add_system_set(
            // every frame whilst counting down
            SystemSet::on_update(
                state::GameState::LevelCountdown,
            )
            // countdown the actual timer
            .with_system(Countdown::countdown.system())
            // tick along the actual timer
            .with_system(Countdown::tick.system()),
        )
        // cleanup countdown timer
        .add_system(state::GameState::despawn::<
            Countdown,
        >(
            state::GameState::LevelCountdown,
        ));
    }

    // spawns in the countdown timer
    fn spawn(
        mut commands: Commands,
        materials: Res<asset::Materials>,
        mut rapier_cofig: ResMut<RapierConfiguration>,
        mut game_state: ResMut<State<state::GameState>>,
    ) {
        // putting another state over the current
        // GameState, just like Pause does
        // to pause the game logic being run
        game_state
            .push(state::GameState::LevelCountdown)
            .unwrap();

        // disabling the external physics
        rapier_cofig.physics_pipeline_active = false;
        // spawning the actual UI element
        commands
            .spawn_bundle(TextBundle {
                text: Text::with_section(
                    Countdown::default().start.to_string(),
                    TextStyle {
                        font: materials.main_font.clone(),
                        font_size: 120.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                ),
                ..Default::default()
            })
            .insert(Countdown::default());
    }

    // advances the countdown timer
    // according to the time kept
    // by the game engine
    fn tick(
        time: Res<Time>,
        mut countdown: Query<&mut Countdown>,
    ) {
        countdown
            .single_mut()
            .unwrap()
            .timer
            .tick(time.delta());
    }

    // dectects when the countdown is finished
    // and otherwise just changes the text to
    // reflect the actual time remaining on the
    // timer stored in countdown
    fn countdown(
        mut commands: Commands,
        mut query: Query<(
            Entity,
            &mut Text,
            &mut Countdown,
        )>,
        mut game_state: ResMut<State<state::GameState>>,
        mut rapier_cofig: ResMut<RapierConfiguration>,
    ) {
        // getting the entitiy and text and countdown struct
        if let Ok((entity, mut text, countdown)) =
            query.single_mut()
        {
            // when the countodown is finsihed
            // despawn the UI element it has as
            // well as returning to the InLevel state
            // and activating the physics
            if countdown.timer.finished() {
                commands.entity(entity).despawn_recursive();
                game_state.pop().unwrap();
                rapier_cofig.physics_pipeline_active = true;
            } else {
                // otherwise update the text using this jank
                text.sections[0].value =
                    // take the remaining percent
                    // on the timer and times that
                    // by the start then round that
                    // then cast it to a u32 and
                    // add one so it's never 0
                    ((countdown.timer.percent_left()
                        * countdown.start)
                        .trunc()
                        as u32
                        + 1)
                    .to_string();
            }
        }
    }
}

// displays item menu
pub struct ItemMenu;

pub struct ItemMenuButton {
    item: item::ItemId,
}

impl ItemMenu {
    // adding item menu's logic to
    // the app
    pub fn build(app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(state::GameState::ItemMenu)
                .with_system(ItemMenu::spawn.system()),
        )
        .add_system_set(
            SystemSet::on_update(
                state::GameState::ItemMenu,
            )
            .with_system(ItemMenu::interactions.system()),
        )
        .add_system(state::GameState::despawn::<
            ItemMenu,
        >(
            state::GameState::ItemMenu
        ));
    }

    // spawn in the item menu
    // for the player to see
    pub fn spawn(
        mut commands: Commands,
        materials: Res<asset::Materials>,
    ) {
        // getting an rng generator
        let mut rng = ThreadRng::default();
        // creates a random item
        macro_rules! rand_item {
            () => {
                item::ItemId::random(&mut rng).to_item()
            };
        }
        // getting three random items
        let items =
            vec![rand_item!(), rand_item!(), rand_item!()];
        // spawning the div that contains the
        // three item buttons
        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(
                        Val::Percent(40.0),
                        Val::Percent(60.0),
                    ),
                    margin: Rect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                material: materials.tile_wall.clone(),
                ..Default::default()
            })
            .insert(ItemMenu)
            .with_children(|parent| {
                // spawning the three buttons
                for item in items.into_iter() {
                    ItemMenu::spawn_button(
                        parent,
                        item,
                        &*materials,
                    )
                }
            });
    }

    // spawns in a button
    // with a given item and
    // as a child of parent
    fn spawn_button(
        parent: &mut ChildBuilder,
        item: Box<dyn item::Item>,
        materials: &asset::Materials,
    ) {
        parent
            .spawn_bundle(ButtonBundle {
                style: Style {
                    size: Size::new(
                        Val::Px(crate::WINDOW_WIDTH * 0.4),
                        Val::Px(crate::WINDOW_HEIGHT * 0.8),
                    ),
                    margin: Rect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: Rect::all(Val::Px(10.0)),
                    flex_direction:
                        FlexDirection::ColumnReverse,
                    ..Default::default()
                },
                material: materials.player_gun.clone(),
                ..Default::default()
            })
            .insert(ItemMenuButton { item: item.id() })
            .with_children(|parent| {
                // item name and description text
                parent.spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: item
                                    .name()
                                    .to_string()
                                    + "\n",
                                style: TextStyle {
                                    font: materials
                                        .main_font
                                        .clone(),
                                    font_size: 14.0,
                                    color: Color::rgb(
                                        0.0, 0.0, 0.0,
                                    ),
                                },
                            },
                            TextSection {
                                value: item
                                    .desc()
                                    .to_string(),
                                style: TextStyle {
                                    font: materials
                                        .main_font
                                        .clone(),
                                    font_size: 10.0,
                                    color: Color::rgb(
                                        0.0, 0.0, 0.0,
                                    ),
                                },
                            },
                        ],
                        alignment: TextAlignment {
                            vertical: VerticalAlign::Top,
                            horizontal:
                                HorizontalAlign::Center,
                        },
                    },
                    style: Style {
                        max_size: Size::new(
                            Val::Px(
                                crate::WINDOW_WIDTH * 0.4,
                            ),
                            Val::Px(
                                crate::WINDOW_HEIGHT * 0.8,
                            ),
                        ),
                        align_items: AlignItems::Center,
                        justify_content:
                            JustifyContent::FlexStart,
                        flex_direction:
                            FlexDirection::ColumnReverse,
                        ..Default::default()
                    },
                    ..Default::default()
                });
            });
    }

    // handles interactions with buttons
    pub fn interactions(
        mut app_state: ResMut<State<state::GameState>>,
        mut items: ResMut<item::ItemManager>,
        query: Query<
            (&Interaction, &ItemMenuButton),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, button) in query.iter() {
            // if a given button is pressed then
            // the item it stores is added to the player
            // and the item menu is closed
            if let Interaction::Clicked = interaction {
                items.add(button.item.clone());
                app_state
                    .overwrite_set(
                        state::GameState::LoadingLevel,
                    )
                    .unwrap();
            }
        }
    }
}
