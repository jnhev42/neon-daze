use super::{
    Difficulty, Grid, GridPos, Tile, TileContent, TileSpawn,
};
use bevy::prelude::*;
use rand::{
    rngs::ThreadRng,
    seq::{IteratorRandom, SliceRandom},
    Rng,
};
use std::ops::Range;

// this holds the data
// for a rectangle on the
// grid that is either
// clear or a wall
#[derive(Debug, Clone)]
struct GridRect {
    start: GridPos,
    end: GridPos,
}

impl GridRect {
    pub const WHOLE_GRID: GridRect = GridRect {
        start: GridPos::MIN,
        end: GridPos::MAX,
    };

    // creates a random rectangle
    // with min size min and max size max
    // that's inside inside
    pub fn random(
        rng: &mut ThreadRng,
        min: GridPos,
        max: GridPos,
        inside: &GridRect,
    ) -> GridRect {
        let start = GridPos::random(
            rng,
            inside.start,
            inside.end - max,
        );
        let end =
            GridPos::random(rng, start + min, start + max);
        Self { start, end }
    }

    // sets all tiles within its bounds to a given tile
    pub fn apply(self, grid: &mut Grid, tile: Tile) {
        grid.apply_in_area_mut(self.start, self.end, |t| {
            *t = tile.clone()
        })
    }
}

#[derive(Debug)]
struct Clearing {
    clearing: GridRect,
    obstacles: Vec<GridRect>,
}

impl Clearing {
    // the range of numbers of obstacles per clearing
    pub const OBSTACLE_RANGE: Range<usize> = 1..3;
    // the minimum size of an obstacle
    pub const OBSTACLE_MIN_SIZE: usize = 1;
    // the minumum size of a clearing
    pub const CLEARING_MIN_SIZE: usize = 4;
    // the range of numbers of clearings per level
    pub const CLEARING_RANGE: Range<usize> = 4..5;

    // creates a random clearing
    pub fn random(rng: &mut ThreadRng) -> Clearing {
        // create a random rect on the grid
        let clearing = GridRect::random(
            rng,
            GridPos::MIN + Self::CLEARING_MIN_SIZE,
            GridPos::MAX - Self::CLEARING_MIN_SIZE,
            &GridRect::WHOLE_GRID,
        );
        // the minimum size of an obstacle
        let obstacle_min = GridPos::new(
            Clearing::OBSTACLE_MIN_SIZE,
            Clearing::OBSTACLE_MIN_SIZE,
        );
        // the maximum size of an obstacle
        let obstacle_max = clearing.end
            - clearing.start
            - Clearing::OBSTACLE_MIN_SIZE;
        // creating a random number of obstacles
        let mut obstacles = Vec::new();
        for _ in 0..rng.gen_range(Clearing::OBSTACLE_RANGE)
        {
            obstacles.push(GridRect::random(
                rng,
                obstacle_min,
                obstacle_max,
                &clearing,
            ))
        }
        // returning the created clearing
        Clearing {
            clearing,
            obstacles,
        }
    }

    // applies a clearing to the grid
    pub fn apply(self, grid: &mut Grid) {
        // setting all the tiles inside the clearing
        // to be empty
        self.clearing.apply(
            grid,
            Tile {
                cont: TileContent::Empty(
                    TileSpawn::Unreachable,
                ),
            },
        );
        // calling all the obstacle applies
        // to set the obstacle areas to walls
        for obstacle in self.obstacles.into_iter() {
            obstacle.apply(
                grid,
                Tile {
                    cont: TileContent::Wall,
                },
            )
        }
    }
}

impl Grid {
    const PLAYER_SPAWN_BUFFER: isize = 2;

    pub fn level_generate(
        mut grid: ResMut<Grid>,
        difficulty: Res<Difficulty>,
    ) {
        let mut rng = ThreadRng::default();
        *grid = Grid::generate(&mut rng, &*difficulty);
    }

    // adds the actual space to the level
    fn add_clearings(&mut self, rng: &mut ThreadRng) {
        // creates a random clearing and then writes it
        // to the grid a random number of times
        for _ in 0..rng.gen_range(Clearing::CLEARING_RANGE)
        {
            Clearing::random(rng).apply(self)
        }
    }

    // adds a player to the level
    fn add_player(&mut self, rng: &mut ThreadRng) {
        // picks a random positon whose tile isn't a wall
        self.player = GridPos::iter_all()
            .filter(|pos| {
                matches!(
                    self[*pos].cont,
                    TileContent::Empty(
                        TileSpawn::Unreachable,
                    )
                )
            })
            .choose(rng);
    }

    fn add_player_spawn_buffer(&mut self) {
        // binding to variables for clarity
        let (x, y) = (
            self.player.unwrap().x as isize,
            self.player.unwrap().y as isize,
        );
        let buffer = Self::PLAYER_SPAWN_BUFFER;
        // applying in a square around the player
        self.apply_in_area_mut(
            GridPos::new_bounded(x - buffer, y - buffer),
            GridPos::new_bounded(x + buffer, y + buffer),
            |tile| {
                // if a tile is empty
                // block spawning on it
                // by marking it as the player's
                // spawn
                if let TileContent::Empty(ref mut spawn) =
                    tile.cont
                {
                    *spawn = TileSpawn::Player
                }
            },
        )
    }

    // makes sure that tiles not reachable from
    // where the player spawns are walls to make
    // level layout easier
    fn block_unreachable_areas(&mut self) {
        // breadth first marking every empty tile
        // as having no allocated spawn
        // if it can be reached from the player's
        // spawn through breadth first search
        self.apply_breadth_mut(
            self.player.unwrap(),
            |tile| {
                matches!(tile.cont, TileContent::Empty(_))
            },
            |tile| {
                // since we have reached it
                // mark unreachable tiles as empty
                tile.cont =
                    TileContent::Empty(TileSpawn::None)
            },
        );
        // any tiles still unreachable
        // can be converted into walls
        // for better visuals
        self.apply_in_area_mut(
            GridPos::MIN,
            GridPos::MAX,
            |tile| {
                if tile.cont
                    == TileContent::Empty(
                        TileSpawn::Unreachable,
                    )
                {
                    tile.cont = TileContent::Wall;
                }
            },
        );
    }

    // adds enemies to the grid
    fn add_enemies(
        &mut self,
        rng: &mut ThreadRng,
        difficulty: &Difficulty,
    ) {
        // initialising the enemy position store
        let mut enemies = Vec::new();
        // getting all the postitons enemies can
        // spawn on in the grid
        let mut spawns = self
            .iter()
            // zipping with grid positions so that
            // can know position from tile
            .zip(GridPos::iter_all())
            // filters out tiles that aren't empty
            // and not already allocated for a spawn
            .filter(|(tile, _)| {
                matches!(
                    tile.cont,
                    TileContent::Empty(TileSpawn::None)
                )
            })
            // removing the actual tile data from
            // spawns to appease the complier
            // and it's fretting about pointer aliasing
            .map(|(_, pos)| pos)
            // collecting them all into a vector
            .collect::<Vec<_>>();
        // shuffles those positions into a random order
        spawns.shuffle(rng);

        // the total number of points that can
        // be spent generating the level
        let mut points = difficulty.points();
        // while there are points left and potential
        // spots enemies can be placed
        while points > 0.0 && !spawns.is_empty() {
            // getting a postiong for the enemy
            let pos = spawns.pop().unwrap();
            // base cost of an enemy placement in points
            let mut cost = 300.0;
            // for each square between the player and enemy
            // the enemy costs five points less to spawn in
            cost -= 5.0
                * self
                    .path_between(
                        self.player.unwrap(),
                        pos,
                        |tile| {
                            matches!(
                                tile.cont,
                                TileContent::Empty(_)
                            )
                        },
                    )
                    .unwrap()
                    .len() as f32;
            // incase the cost becomes negative
            // clamp it at 50.0 per enemy
            if cost < 50.0 {
                cost = 50.0;
            }
            // if the enemy can see the player
            // from their starting position
            // double their cost as this enemy
            // will be attacking immediately
            let filter = |tile: &Tile| {
                matches!(
                    tile.cont,
                    TileContent::Empty(_)
                )
            };
            if self.line_of_sight(
                self.player.unwrap(),
                pos,
                filter,
            ) {
                cost *= 2.0;
            }
            // if there's enough points to place this enemy
            // then charge that amount of points and record it's
            // position
            if cost < points {
                points -= cost;
                enemies.push(pos);
            }
        }
        // marking all the tiles on the grid where
        // enemies will spawn as such
        for pos in enemies.iter() {
            self[*pos].cont =
                TileContent::Empty(TileSpawn::Enemy)
        }
        // writing this new array to the grid struct
        self.enemies = enemies;
    }

    pub fn generate(
        rng: &mut ThreadRng,
        difficulty: &Difficulty,
    ) -> Grid {
        // calls continue if the expression passed
        // evaluates to true
        macro_rules! restart_if {
            ($b:expr) => {
                if $b {
                    continue;
                }
            };
        }
        // declaring grid's scope outside of
        // the loops so it can be returned
        let mut grid;
        loop {
            // creating a new grid of entirely walls
            grid = Grid::default();
            // adding in clearings and the obstacles inside them
            grid.add_clearings(rng);
            // picking a spawn for the player
            grid.add_player(rng);
            // if the player couldn't find anywhere
            // to be placed restart the process
            restart_if!(grid.player.is_none());
            // blocking any areas the player can't reach
            grid.block_unreachable_areas();
            // adds a protected area around where the player
            // spawns so there are no enemies near them
            grid.add_player_spawn_buffer();
            // restarting if less than a third of the
            // grid is empty as this will be too small
            // of a level
            restart_if!(
                grid.iter()
                    .filter(|tile| {
                        matches!(
                            tile.cont,
                            TileContent::Empty(_)
                        )
                    })
                    .count()
                    < (Grid::WIDTH * Grid::HEIGHT) / 3
            );
            // spawning enemies on the grid
            grid.add_enemies(rng, difficulty);
            restart_if!(grid.enemies.is_empty());
            // all restart_if s passed so break out of
            // loop
            break;
        }
        // returning the generated grid
        grid
    }
}
