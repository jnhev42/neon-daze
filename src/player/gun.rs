use crate::{
    asset, cooldown, grid, item, player::bullet, state,
};
use bevy::prelude::{DespawnRecursiveExt, *};
use bevy_rapier2d::prelude::*;
use core::f32::consts::PI;

pub struct GunPlugin;

impl Plugin for GunPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_update(state::GameState::InLevel)
                .with_system(Gun::point.system())
                .with_system(Gun::shoot.system())
                .with_system(Gun::bullet_lifetime.system())
                .with_system(
                    Gun::bullet_collisions.system(),
                ),
        )
        // cleaning up all the bullets at the end of the level
        .add_system(state::GameState::despawn::<
            bullet::Bullet,
        >(
            state::GameState::InLevel
        ));
    }
}
// stores all data important to
// the gun and it shooting
pub struct Gun {
    dir_rad: f32,
    bullets: Vec<Entity>,
}

impl Gun {
    // points the gun towards the mouse cursor
    // both onscreen and in game logic
    pub fn point(
        windows: Res<Windows>,
        mut queries: QuerySet<(
            Query<&Transform, With<crate::MainCamera>>,
            Query<(
                &mut Transform,
                &GlobalTransform,
                &mut Gun,
            )>,
        )>,
    ) {
        // get the primary window
        let window = windows.get_primary().unwrap();
        // get the size of the window
        let size = Vec2::new(
            window.width() as f32,
            window.height() as f32,
        );
        // get the position of the cursor
        let mouse_pos = match window.cursor_position() {
            Some(pos) => pos,
            // if it has no position
            // then return early
            // and leave it pointing the
            // way it was as the cursor is
            // off the window
            None => return,
        };

        // the default orthographic projection is in pixels from the center
        // so translate it from px from top right to from center
        let screen_pos = mouse_pos - size / 2.0;

        // getting the position of the camera
        let camera_transform =
            queries.q0().single().unwrap();

        // transforms the screen_pos into a world pos with
        let pos_wld = camera_transform.compute_matrix()
            * screen_pos.extend(0.0).extend(1.0);

        // write our mouses xy into the game
        let target: Vec2 = pos_wld.truncate().truncate();

        let (
            mut gun_transform,
            global_gun_transform,
            mut gun,
        ) = queries.q1_mut().single_mut().unwrap();

        let gun_pos: Vec2 =
            global_gun_transform.translation.truncate();
        // target - pos is the vector that indicates
        // distance between target and position then it gets the
        // angle between it and "north" however this returns in
        // the range -PI/2 to PI/2 which we need to change
        // into not being negative for the game engines
        // rotation system
        let angle = (2. * PI)
            - (target - gun_pos).angle_between(Vec2::Y);
        // finally writing the calculated angle to the
        // sprite and our own internal logic for later use
        gun_transform.rotation =
            Quat::from_rotation_z(angle);
        gun.dir_rad = angle;
    }

    // detects when the player is shooting and spawns
    // a projectile
    pub fn shoot(
        mut commands: Commands,
        mouse_buttons: Res<Input<MouseButton>>,
        mut gun_query: Query<(
            &GlobalTransform,
            &mut cooldown::Cooldown,
            &mut Gun,
        )>,
        config: Res<item::Config>,
    ) {
        // mouse has not just been pressed
        // so the player is not trying to shoot
        // so early return
        if config.flags.contains(&item::ItemId::AutoFire) {
            if !mouse_buttons.pressed(MouseButton::Left) {
                return;
            }
        } else if !mouse_buttons
            .just_pressed(MouseButton::Left)
        {
            return;
        }
        // getting the gun's position and the cooldown on its
        // use as well as the actual gun itself
        let (gun_pos, mut cooldown, mut gun) =
            gun_query.single_mut().unwrap();
        if cooldown.is_over() {
            let angle = gun.dir_rad
                + (rand::random::<f32>() - 0.5)
                    * 2.0
                    * config.gun.deviation;
            // spawns a bullet and grabs it's id
            let id = commands
                .spawn_bundle(bullet::BulletBundle::new(
                    config.gun.clone(),
                    angle,
                    gun_pos.translation.truncate(),
                ))
                .id();
            // stores that bullets id and restarts cooldown
            gun.bullets.push(id);
            cooldown.reset();
        }
    }

    // despawns bullets after their cooldown is over
    pub fn bullet_lifetime(
        mut commands: Commands,
        lifetimes: Query<&cooldown::Cooldown>,
        mut gun: Query<&mut Gun>,
    ) {
        // deletes all elemets of the list
        // that don't return true for
        // the given predicate
        gun.single_mut().unwrap().bullets.retain(
            |&bullet| {
                let lifetime = match lifetimes.get(bullet) {
                    Ok(c) => c,
                    Err(_) => return true,
                };
                if lifetime.is_over() {
                    // the bullets lifetime is over
                    // so despawn it
                    commands
                        .entity(bullet)
                        .despawn_recursive();
                    false
                } else {
                    true
                }
            },
        );
    }

    // handles the bullets colliding with things
    pub fn bullet_collisions(
        mut commands: Commands,
        walls: Query<Entity, With<grid::Tile>>,
        mut gun: Query<&mut Gun>,
        mut contact_events: EventReader<ContactEvent>,
        config: Res<item::Config>,
    ) {
        // if the bullets are bouncy just don't do collisions
        if config.flags.contains(&item::ItemId::Bouncy) {
            return;
        }
        // getting the gun struct
        let mut gun = gun.single_mut().unwrap();
        // iterating over all the contanct events
        for contact in contact_events.iter() {
            // only dealing with initial collisions
            if let ContactEvent::Started(h1, h2) = contact {
                // getting the game entities of
                // the colliding entities
                let (e1, e2) = (h1.entity(), h2.entity());
                // no garunteed ordering of colliders
                // so checking if either is a
                // wall / bullet
                for (bullet, wall) in
                    [(e1, e2), (e2, e1)].iter()
                {
                    // checks that the wall is a wall
                    // and that the bullet is one
                    // owned by gun
                    if let (Ok(_), Some(idx)) = (
                        walls.get(*wall),
                        gun.bullets
                            .iter()
                            .position(|i| i == bullet),
                    ) {
                        // for now if a bullet hits a wall
                        // it disappears
                        commands
                            .entity(
                                gun.bullets
                                    .swap_remove(idx),
                            )
                            .despawn_recursive();
                    }
                }
            }
        }
    }
}

// all the components the gun
// needs to work
#[derive(Bundle)]
pub struct GunBundle {
    gun: Gun,
    cooldown: cooldown::Cooldown,
    #[bundle]
    sprite: SpriteBundle,
}

impl GunBundle {
    // creates a new gun bundle
    pub fn new(builder: GunBuilder) -> GunBundle {
        let mut cooldown =
            cooldown::Cooldown::new(Some(builder.cooldown));
        cooldown.set_elapsed(builder.cooldown - 0.01);
        GunBundle {
            gun: Gun {
                dir_rad: 0.0,
                bullets: Vec::new(),
            },
            cooldown,
            sprite: SpriteBundle {
                sprite: Sprite::new(builder.size),
                material: builder.material,
                transform: Transform::from_translation(
                    Vec2::ZERO.extend(5.0),
                ),
                ..Default::default()
            },
        }
    }
}

// this holds all the data that the gun
// needs to build itself and projectiles
// at runtime
#[derive(Debug, Clone)]
pub struct GunBuilder {
    pub size: Vec2,
    pub cooldown: f32,
    pub material: Handle<ColorMaterial>,
    pub deviation: f32,
    pub lifetime: f32,
    pub speed: f32,
}

impl FromWorld for GunBuilder {
    // returns a default version of the GunBuilder
    fn from_world(world: &mut World) -> Self {
        let materials = world
            .get_resource::<asset::Materials>()
            .unwrap();
        Self {
            size: Vec2::new(5.0, 10.0),
            cooldown: 0.3,
            material: materials.player_gun.clone(),
            deviation: 0.1,
            lifetime: 1.0,
            speed: 500.,
        }
    }
}
