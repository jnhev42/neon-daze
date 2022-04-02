use crate::{asset, grid, phys, player, state};
use bevy::{prelude::DespawnRecursiveExt, prelude::*};
use bevy_rapier2d::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    // initialising all the enemys associated systems
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(state::GameState::InLevel)
                .with_system(Enemy::spawn.system()),
        )
        .add_system_set(
            SystemSet::on_update(state::GameState::InLevel)
                .with_system(Enemy::path.system())
                .with_system(Enemy::collide.system())
                .with_system(Enemy::check_cleared.system()),
        )
        .add_system(state::GameState::despawn::<
            Enemy,
        >(
            state::GameState::InLevel
        ));
    }
}

pub struct Enemy {
    target: Option<Vec2>,
}

impl Enemy {
    // spawns in every enemy according to where the
    // grid says they should be
    pub fn spawn(
        mut commands: Commands,
        grid: Res<grid::Grid>,
        materials: Res<asset::Materials>,
    ) {
        // using batched spawning to speed up
        // the level load
        commands.spawn_batch(
            grid.enemies
                .iter()
                .map(|pos| {
                    EnemyBundle::new(
                        pos.to_world(),
                        &materials,
                    )
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn path(
        enemies: Query<
            (
                &mut RigidBodyVelocity,
                &RigidBodyPosition,
                &mut Enemy,
            ),
            With<Enemy>,
        >,
        player: Query<
            (Entity, &Transform),
            With<player::Player>,
        >,
        phys: Res<QueryPipeline>,
        collider_query: QueryPipelineColliderComponentsQuery,
    ) {
        // getting the players
        let (player_id, player) = player.single().unwrap();
        // geting the players position
        let player = player.translation.truncate();
        let collider_set =
            QueryPipelineColliderComponentsSet(
                &collider_query,
            );
        enemies.for_each_mut(|(mut vel, pos, mut enemy)| {
            let pos: Vec2 = pos.position.translation.into();
            // if the enemy has reached their target
            // then stop trying to path to it
            if let Some(target) = enemy.target {
                if pos.abs_diff_eq(target, 4.0) {
                    enemy.target = None;
                }
            }
            // casting a ray between the player and the enemy
            // pretty jank api i agree
            if let Some((handle, _)) = phys.cast_ray(
                &collider_set,
                // starting from my position
                // and going to
                &Ray::new(
                    pos.into(),
                    (player - pos).normalize().into(),
                ),
                // maximum time of impact for the ray
                // for now the enemies have
                // infinite range vision
                Real::MAX,
                true,
                // limit vision to only certain objects
                phys::masks::enemy_vision(),
                None,
            ) {
                // the handle is the first thing that the
                // ray hit when being cast, this tests
                // whether the first thing the ray hit was
                // the player in which case set the
                // target to the player's position
                if handle.entity() == player_id {
                    enemy.target = Some(player);
                }
                // no else clause as otherwise
                // line of sight to the player is blocked
                // by a wall and as such the enemy
                // must either continue going towards
                // where it last saw the player or
                // if it hasn't seen the player or
                // has reached where it last saw the
                // player stand still

                // calculating the direction that the
                // enemy should move in
                vel.linvel =
                    // if the enemy has a target then move 
                    // in the direction that target is  
                    // relativeto itself at a 
                    // speed of 250px per second
                    if let Some(target) = enemy.target {
                        ((target - pos).normalize() * 250.0)
                            .into()
                    } else {
                        // if the enemy has no target
                        // then just don't move
                        Vec2::ZERO.into()
                    }
            }
        })
    }

    pub fn collide(
        mut commands: Commands,
        mut contact_events: EventReader<ContactEvent>,
        enemies: Query<(), With<Enemy>>,
        bullets: Query<(), With<player::bullet::Bullet>>,
    ) {
        for contact in contact_events.iter() {
            // only dealing with initial collisions
            if let ContactEvent::Started(h1, h2) = contact {
                // getting the entity handles
                // from the physics handles
                let (e1, e2) = (h1.entity(), h2.entity());
                // checking both
                // that e1 is an enemy or bullet
                // and that e2 is an enemy or bullet
                for (bullet, enemy) in [(e1, e2), (e2, e1)]
                {
                    // if the bullet is a bullet and the
                    // enemy is an enemy
                    if enemies.get(enemy).is_ok()
                        && bullets.get(bullet).is_ok()
                    {
                        // despawn the bullet
                        // and the enemy
                        commands
                            .entity(bullet)
                            .despawn_recursive();
                        commands
                            .entity(enemy)
                            .despawn_recursive();
                        break;
                    }
                }
            }
        }
    }

    // checks to see if there are no more enemies
    // on the level
    pub fn check_cleared(
        enemies: Query<(), With<Enemy>>,
        mut game_events: EventWriter<state::GameEvent>,
    ) {
        if matches!(enemies.iter().next(), None) {
            game_events.send(state::GameEvent::LevelClear)
        }
    }
}

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    sync: ColliderPositionSync,
    #[bundle]
    collider: ColliderBundle,
    #[bundle]
    rigid_body: RigidBodyBundle,
    #[bundle]
    sprite: SpriteBundle,
}

impl EnemyBundle {
    // creates a new enemy entity with
    // all the required components
    pub fn new(
        pos: Vec2,
        materials: &asset::Materials,
    ) -> Self {
        Self {
            enemy: Enemy { target: None },
            sync: ColliderPositionSync::Discrete,
            collider: ColliderBundle {
                shape: ColliderShape::cuboid(10.0, 10.0),
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
                    collision_groups: phys::masks::enemy(),
                    ..Default::default()
                },
                ..Default::default()
            },
            rigid_body: RigidBodyBundle {
                position: pos.into(),
                mass_properties:
                    RigidBodyMassPropsFlags::ROTATION_LOCKED
                        .into(),
                ..Default::default()
            },
            sprite: SpriteBundle {
                transform: Transform::from_translation(
                    pos.extend(4.0),
                ),
                material: materials.enemy.clone(),
                sprite: Sprite::new(Vec2::new(20.0, 20.0)),
                ..Default::default()
            },
        }
    }
}
