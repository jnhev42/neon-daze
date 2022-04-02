use bevy::{app::Events, prelude::*};
use bevy_rapier2d::prelude::*;
// importing state module here
// importing asset module here
use crate::{asset, enemies, grid, item, phys, state};
// this class has no internal data and only
// "inherits" (not how Rust's traits (abstract base classes)
// actually work) Plugin which has the method build
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    // this takes a mutable reference to the
    // result of App::build() that was called in main
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            // this means Player::movement will run every
            // frame when the game state is GameState::InLevel
            SystemSet::on_update(state::GameState::InLevel)
                .with_system(Player::movement.system())
                .with_system(
                    Player::detect_enemy_hits.system(),
                ),
        )
        .add_system_set(
            // this means Player::spawn will run when
            // the GameState::InLevel is first entered
            // (before update is called)
            SystemSet::on_enter(state::GameState::InLevel)
                .with_system(Player::spawn.system()),
        )
        .add_system(Lives::on_hit.system())
        .add_system(state::GameState::despawn::<Player>(
            state::GameState::InLevel,
        ))
        .add_plugin(gun::GunPlugin)
        .init_resource::<Lives>();
    }
}

// this struct holds all the player's data
// (for now there isn't any)
#[derive(Debug, Clone)]
pub struct Player {
    speed: f32,
}

// this is how Rust denotes
// methods being implemented
// on structs (reads Classes)
// it allows for a structs methods
// to be defined in more than one place
impl Player {
    // spawns in the player
    // ResMut<Assets<ColorMaterial>>> is a mutable reference to the
    // colour material asset server which manages loading materials
    // its a little overkill for this case but generally it allows
    // the loading of more complex materials
    fn spawn(
        mut commands: Commands,
        config: Res<item::Config>,
        grid: Res<grid::Grid>,
    ) {
        // spawns in the player
        commands
            .spawn_bundle(PlayerBundle::new(
                grid.player.unwrap().to_world(),
                config.player.clone(),
            ))
            // adds the gun as child of the
            // player
            .with_children(|child| {
                child.spawn_bundle(gun::GunBundle::new(
                    config.gun.clone(),
                ));
            });
    }
    // moves the player around
    // keys tells me what keys on the keyboard are pressed at any given time
    fn movement(
        mut query: Query<(&mut RigidBodyVelocity, &Player)>,
        keys: Res<Input<KeyCode>>,
    ) {
        let mut dir = Vec2::ZERO;
        // move up
        if keys.pressed(KeyCode::W) {
            dir.y += 1.0
        }
        // move left
        if keys.pressed(KeyCode::A) {
            dir.x -= 1.0
        }
        // move down
        if keys.pressed(KeyCode::S) {
            dir.y -= 1.0
        }
        // move right
        if keys.pressed(KeyCode::D) {
            dir.x += 1.0
        }

        // this gets a mutable reference to the players transform
        // and a immutable reference to Player
        // it will crash if this is run when there is not 1 player
        let (mut vel, player) = query.single_mut().unwrap();
        // times direction by speed to get veloctiy
        // normalising it makes it's magnitude
        // unit length. then multiplying by
        // player.speed garuntees its the right length
        // let vel = dir.normalize_or_zero() * player.speed;
        // or the faster version
        // because it can make a bunch of
        // assumptions that an external call
        // can't make as its for more generalised
        // usage whereas this is just faster in
        // this particular case
        // also in Rust the last line of a
        // code block is implicitly returned
        let new_vel = if dir.x == 0.0 || dir.y == 0.0 {
            // if it's got one dimension that's 0
            // then its already normalised
            dir * player.speed
        } else {
            // sqrt(1^2 + 1^2) / (1 + 1)
            // = sqrt(2) / 2
            // as thats total magnitude
            // (pythagoras so root 2)
            // divided by the fraction the x and y
            // components represent but here they're
            // both always 1 so there 1/2
            const NORMALIZE_FACTOR: f32 = 0.70710678118;
            dir * NORMALIZE_FACTOR * player.speed
        };
        // adding the current velocity to the player's translation
        // need to extend it so it has a z which represents what
        // layer the player is on
        vel.linvel = new_vel.into();
    }

    // this detects any collisions between the player
    // and enemies and if there are any sends
    // an event to update anything that should
    // react to the player being hit
    pub fn detect_enemy_hits(
        mut contact_events: EventReader<ContactEvent>,
        player: Query<Entity, With<Player>>,
        enemies: Query<(), With<enemies::Enemy>>,
        mut game_events: EventWriter<state::GameEvent>,
    ) {
        for event in contact_events.iter() {
            if let ContactEvent::Started(h1, h2) = event {
                // getting the entities related to
                // the physics handles of the two contacting things
                let (e1, e2) = (h1.entity(), h2.entity());
                // the engine gives no particular order
                // so test both orders
                for (plr, enemy) in [(e1, e2), (e2, e1)] {
                    if player.get(plr).is_ok()
                        && enemies.get(enemy).is_ok()
                    {
                        // the two contacts were a player
                        // and enemy so the player was hit
                        game_events.send(
                            state::GameEvent::PlayerHit,
                        );
                    }
                }
            }
        }
    }
}

// this groups together components into bundles (read entities)
#[derive(Bundle)]
struct PlayerBundle {
    // this holds all the player's data
    player: Player,
    // this unpacks the SpriteBundle of components
    // and and add all of them to the player
    #[bundle]
    sprite: SpriteBundle,
    // adding the components that
    // link the player bundle
    // to the physics system
    sync: ColliderPositionSync,
    #[bundle]
    collider: ColliderBundle,
    #[bundle]
    rigid_body: RigidBodyBundle,
}

impl PlayerBundle {
    // creates a new player along with all
    // the players associated components
    // (so far just a sprite)
    fn new(
        pos: Vec2,
        builder: PlayerBuilder,
    ) -> PlayerBundle {
        PlayerBundle {
            player: Player {
                speed: builder.speed,
            },
            sprite: SpriteBundle {
                // makes the sprite white
                material: builder.material.clone(),
                // makes the sprite a 10 * 10 square
                sprite: Sprite::new(builder.size),
                transform: Transform::from_translation(
                    pos.extend(2.0),
                ),
                ..Default::default()
            },
            collider: ColliderBundle {
                shape: ColliderShape::cuboid(
                    builder.size.x / 2.,
                    builder.size.y / 2.,
                ),
                // this is so that the player doesn't experience friction
                material: ColliderMaterial {
                    restitution: 0.0,
                    friction: 0.0,
                    friction_combine_rule:
                        CoefficientCombineRule::Min,
                    restitution_combine_rule:
                        CoefficientCombineRule::Min,
                    ..Default::default()
                },
                flags: ColliderFlags {
                    collision_groups: phys::masks::player(),
                    active_events:
                        ActiveEvents::CONTACT_EVENTS,
                    ..Default::default()
                },
                ..Default::default()
            },
            // the physics body of the player
            rigid_body: RigidBodyBundle {
                position: pos.into(),
                // stops the player spinning off axis
                mass_properties:
                    RigidBodyMassPropsFlags::ROTATION_LOCKED
                        .into(),
                ..Default::default()
            },
            sync: ColliderPositionSync::Discrete,
        }
    }
}

// this holds the data that interfaces the
// items and the player's behaviour
#[derive(Debug, Clone)]
pub struct PlayerBuilder {
    pub speed: f32,
    pub material: Handle<ColorMaterial>,
    pub size: Vec2,
}

impl FromWorld for PlayerBuilder {
    // returns a default version of the player builder
    fn from_world(world: &mut World) -> Self {
        let materials = world
            .get_resource::<asset::Materials>()
            .unwrap();
        Self {
            speed: 200.0,
            material: materials.player_body.clone(),
            size: Vec2::new(20., 20.),
        }
    }
}

// this stores how many lives the player has
#[derive(Debug)]
pub struct Lives(pub u32);

impl Default for Lives {
    // the starting amount of lives is 5
    fn default() -> Self {
        Self(5)
    }
}

impl Lives {
    // when the player is hit this is called
    pub fn on_hit(
        mut lives: ResMut<Lives>,
        mut game_events: ResMut<Events<state::GameEvent>>,
    ) {
        // getting a way of reading the evens
        let mut reader = game_events.get_reader();
        // if any of the events are the player getting hit
        if reader.iter(&*game_events).any(|event| {
            matches!(event, state::GameEvent::PlayerHit)
        }) {
            // if the player has one life then
            // this is their last life so game
            // over
            if lives.0 <= 1 {
                game_events
                    .send(state::GameEvent::GameOver);
                // resetting the player's lives counter
                *lives = Lives::default();
            } else {
                // otherwise subtract one
                // from the player's lives
                lives.0 -= 1;
            }
        }
    }
}

// adding submodules to hold code for
// the gun and bullets
mod gun;
pub use gun::GunBuilder;

pub mod bullet;
