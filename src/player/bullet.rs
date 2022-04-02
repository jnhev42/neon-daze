use crate::{cooldown, just_spawned, phys};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::f32::consts::PI;

// marker struct to make the bullet more
// identifiable, in future will be used
// for something probably
pub struct Bullet;

// holds all the components
// that make up a bullet
#[derive(Bundle)]
pub struct BulletBundle {
    // how long the bullet exists for
    lifetime: cooldown::Cooldown,
    just_spawned: just_spawned::JustSpawned,
    bullet: Bullet,
    sync: ColliderPositionSync,
    #[bundle]
    sprite: SpriteBundle,
    #[bundle]
    rigid_body: RigidBodyBundle,
    #[bundle]
    collider: ColliderBundle,
}

impl BulletBundle {
    // creates a new bullet bundle
    // moving at a given angle and position
    pub fn new(
        builder: super::GunBuilder,
        angle: f32,
        pos: Vec2,
    ) -> Self {
        BulletBundle {
            bullet: Bullet,
            just_spawned: just_spawned::JustSpawned,
            sync: ColliderPositionSync::Discrete,
            lifetime: cooldown::Cooldown::new(Some(
                builder.lifetime,
            )),
            sprite: SpriteBundle {
                material: builder.material.clone(),
                transform: Transform::from_translation(
                    pos.extend(5.0),
                ),
                visible: Visible {
                    is_visible: false,
                    ..Default::default()
                },
                sprite: Sprite::new(builder.size),
                ..Default::default()
            },
            rigid_body: RigidBodyBundle {
                position: (pos, angle).into(),
                velocity: RigidBodyVelocity {
                    // using basic trigonometry to
                    // calculate the velocty the projectile
                    // should move at
                    linvel: Vec2::new(
                        500. * (angle + 0.5 * PI).cos(),
                        500. * (angle + 0.5 * PI).sin(),
                    )
                    .into(),
                    angvel: 0.0,
                },
                // ccd so we don't phase through walls
                ccd: RigidBodyCcd {
                    ccd_enabled: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            collider: ColliderBundle {
                shape: ColliderShape::cuboid(
                    builder.size.x / 2.0,
                    builder.size.y / 2.0,
                ),
                // making the bullet bouncy
                material: ColliderMaterial {
                    restitution: 1.0,
                    friction: 0.0,
                    ..Default::default()
                },
                flags: ColliderFlags {
                    collision_groups:
                        phys::masks::player_bullet(),
                    active_events:
                        ActiveEvents::CONTACT_EVENTS,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
