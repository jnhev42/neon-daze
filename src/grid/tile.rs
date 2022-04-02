use super::GridPos;
use crate::{asset, grid, phys};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// represents one square on the grid
#[derive(Debug, Clone)]
pub struct Tile {
    pub cont: TileContent,
}

// defaults to being empty and
// having no spawn
impl Default for Tile {
    fn default() -> Self {
        Self {
            cont: TileContent::Wall,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum TileContent {
    Empty(TileSpawn),
    Wall,
}

// stores what should spawn on a tile
#[derive(PartialEq, Debug, Clone)]
pub enum TileSpawn {
    Unreachable,
    None,
    Blocked,
    Player,
    Enemy,
}

// for the creation of tile entities
// there needs to be a bundle of tile
// components
#[derive(Bundle)]
pub struct TileBundle {
    tile: Tile,
    #[bundle]
    sprite: SpriteBundle,
    // linking the tile to the
    // physics system
    sync: ColliderPositionSync,
    #[bundle]
    collider: ColliderBundle,
}

impl TileBundle {
    // creates a new tile bundle
    pub fn new(
        tile: &Tile,
        pos: &GridPos,
        materials: &asset::Materials,
    ) -> Self {
        Self {
            tile: tile.clone(),
            sprite: SpriteBundle {
                // giving a sprite sized relative to the window
                sprite: Sprite::new(Vec2::new(
                    crate::WINDOW_WIDTH
                        / super::Grid::WIDTH as f32,
                    crate::WINDOW_HEIGHT
                        / super::Grid::HEIGHT as f32,
                )),
                // material means color so im matching
                // against Wall and Empty for different
                // colors
                material: match tile.cont {
                    TileContent::Wall => {
                        materials.tile_wall.clone()
                    }
                    TileContent::Empty(_) => {
                        materials.tile_empty.clone()
                    }
                },
                // setting the position of the tile
                // on the screen with its grid position
                // translated into a world coordinate
                transform: Transform::from_translation(
                    pos.to_world().extend(0.0),
                ),

                ..Default::default()
            },
            // syncs the tile's transform with its
            // position in the physics engine
            sync: ColliderPositionSync::Discrete,
            // the set of components that make the colldier
            // collide like a wall
            collider: ColliderBundle {
                // the shape of the tile needs to match it's
                // sprite but rapier's cuboids are measured
                // by extent which is half width/height
                shape: ColliderShape::cuboid(
                    crate::WINDOW_HEIGHT
                        / grid::Grid::HEIGHT as f32
                        / 2.0,
                    crate::WINDOW_WIDTH
                        / grid::Grid::WIDTH as f32
                        / 2.0,
                ),
                position: pos.to_world().into(),
                flags: ColliderFlags {
                    collision_groups: match tile.cont {
                        // if the tile is empty it collides with nothing
                        TileContent::Empty(_) => {
                            phys::masks::none()
                        }
                        // if the tile is a wall it will collide with the
                        // player
                        TileContent::Wall => {
                            phys::masks::wall()
                        }
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
