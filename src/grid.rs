use crate::{asset, phys, state};
use bevy::prelude::*;
use bevy_rapier2d::{na::Point2, prelude::*};
use std::{
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // adding the grid as a resource any system can access
            .init_resource::<Grid>()
            // making the grid spawn at the start of every level
            .add_system_set(
                SystemSet::on_enter(
                    state::GameState::InLevel,
                )
                .with_system(Grid::spawn.system())
                .with_system(Walls::spawn.system()),
            )
            .add_system(state::GameState::despawn::<Tile>(
                state::GameState::InLevel,
            ))
            .add_system(state::GameState::despawn::<Walls>(
                state::GameState::InLevel,
            ))
            .add_system(
                Grid::level_generate
                    .system()
                    .with_run_criteria(State::<
                        state::GameState,
                    >::on_enter(
                        state::GameState::LoadingLevel,
                    )),
            )
            .add_system(
                leave_on_load.system().with_run_criteria(
                    State::<state::GameState>::on_update(
                        state::GameState::LoadingLevel,
                    ),
                ),
            )
            .init_resource::<Difficulty>()
            .add_system_set(
                SystemSet::new()
                    .with_system(
                        Difficulty::increment_level
                            .system(),
                    )
                    .with_system(
                        Difficulty::reset.system(),
                    ),
            );
    }
}

// leaves the loading state when the level loads
fn leave_on_load(
    mut game_state: ResMut<State<state::GameState>>,
) {
    game_state.set(state::GameState::InLevel).unwrap();
}

#[derive(Debug)]
pub struct Grid {
    tiles: Vec<Vec<Tile>>,
    pub player: Option<GridPos>,
    pub enemies: Vec<GridPos>,
}

impl Default for Grid {
    // sets the grid to being completely empty
    fn default() -> Self {
        Self {
            // tiles is a 2d array of Tile
            tiles: vec![
                vec![Tile::default(); Grid::HEIGHT];
                Grid::WIDTH
            ],
            player: None,
            enemies: vec![],
        }
    }
}

impl Grid {
    // this is the compile time
    // garunteed height and width
    // of the grid
    pub const WIDTH: usize = 20;
    pub const HEIGHT: usize = 20;
    // whilst locking this at compile time
    // reduces flexibility, it massively
    // simplifies programming and speeds
    // up performance hugely

    // this system spawns in the grid
    pub fn spawn(
        mut commands: Commands,
        grid: Res<Grid>,
        materials: Res<asset::Materials>,
    ) {
        // spawning in all the tile entities as an
        // iterator because that runs slightly better
        commands.spawn_batch(
            // iterating over every position in the grid
            // and creating a new tile that corresponds to it
            GridPos::iter_all()
                .map(|pos| {
                    TileBundle::new(
                        &grid[pos],
                        &pos,
                        &*materials,
                    )
                })
                // this then needs to be collected into
                // a vector for complicated reasons
                .collect::<Vec<_>>(),
        );
    }

    // runs the function given as the last
    // argument `f` on every tile in a rectangle
    // with top left of start and
    // bottom right of end
    // will be useful later when
    // generating levels
    #[allow(dead_code)]
    fn apply_in_area<T>(
        &self,
        start: GridPos,
        end: GridPos,
        mut f: T,
    ) where
        T: FnMut(&Tile),
    {
        for y in start.y..=end.y {
            for x in start.x..=end.x {
                f(&self.tiles[y][x])
            }
        }
    }
    // the same as the above
    // but it can mutate the state of the
    // tiles its accessing
    fn apply_in_area_mut<T>(
        &mut self,
        start: GridPos,
        end: GridPos,
        mut f: T,
    ) where
        T: FnMut(&mut Tile),
    {
        for y in start.y..=end.y {
            for x in start.x..=end.x {
                f(&mut self.tiles[y][x])
            }
        }
    }

    // searches through all tiles
    // breadth - first
    fn apply_breadth_mut<T, U>(
        &mut self,
        start: GridPos,
        // if this is true then a tile can be pathed through
        mut filter: T,
        mut apply: U,
    ) where
        T: FnMut(&Tile) -> bool,
        U: FnMut(&mut Tile),
    {
        // holds the current positions that are being searched
        let mut current = vec![start];
        // holds the next positions to be scanned
        // (these are adjacent to the current positions)
        let mut next = Vec::new();
        // holds whether a given position has been scanned
        let mut scanned = HashSet::new();
        while !current.is_empty() {
            // removes the elements of current and iterating over them
            for pos in current.drain(0..) {
                // if this tile should be filtered
                // skip to next iteration
                if !filter(&self[pos]) {
                    continue;
                }
                // apply whatever function was passed
                apply(&mut self[pos]);
                // iterate over all the direct adjacent
                // changes in x and y from our current
                // position
                for (xd, yd) in
                    [(1, 0), (-1, 0), (0, 1), (0, -1)]
                {
                    // tries to create a new position offset
                    // by xd and yd
                    if let Some(next_pos) = GridPos::try_new(
                        pos.x as isize + xd,
                        pos.y as isize + yd,
                    ) {
                        // returns false if this tile already scanned
                        // otherwise inserts and then pushes this pos
                        // to the next positions
                        if scanned.insert(next_pos) {
                            next.push(next_pos)
                        }
                    }
                }
            }
            // empties everthing in next into current
            current.append(&mut next);
        }
    }

    // uses bfs to find a path between two points
    // on the grid
    pub fn path_between<T>(
        &self,
        start: GridPos,
        end: GridPos,
        mut filter: T,
    ) -> Option<Vec<GridPos>>
    where
        T: FnMut(&Tile) -> bool,
    {
        // holds the current positions that are being searched
        let mut current = vec![start];
        // holds the next positions to be scanned
        // (these are adjacent to the current positions)
        let mut next = Vec::new();
        // holds whether a given position has been scanned
        let mut scanned = HashSet::new();
        let mut origins = HashMap::new();
        let mut found = false;
        'outer: while !current.is_empty() {
            // removes the elements of current and iterating over them
            for pos in current.drain(0..) {
                // if this tile should be filtered
                // skip to next iteration
                if !filter(&self[pos]) {
                    continue;
                }
                // iterate over all the direct adjacent
                // changes in x and y from our current
                // position
                for (xd, yd) in
                    [(1, 0), (-1, 0), (0, 1), (0, -1)]
                {
                    // tries to create a new position offset
                    // by xd and yd
                    if let Some(next_pos) = GridPos::try_new(
                        pos.x as isize + xd,
                        pos.y as isize + yd,
                    ) {
                        // returns false if this tile already scanned
                        // otherwise inserts and then pushes this pos
                        // to the next positions and maps it's origin
                        // as being from the current pos
                        if scanned.insert(next_pos) {
                            next.push(next_pos);
                            origins.insert(next_pos, pos);
                        }
                    }
                }
                // if the end has been found exit
                if pos == end {
                    found = true;
                    break 'outer;
                }
            }
            // empties everthing in next into current
            current.append(&mut next);
        }
        if found {
            // creating a new vector to store the path
            let mut path = Vec::new();
            // starting from the end point
            let mut current = end;
            // trace the origin of the end to the start
            while current != start {
                path.push(current);
                current = *origins.get(&current).unwrap()
            }
            path.push(current);
            // then reverse the path to get it in the
            // correct order
            path.reverse();
            Some(path)
        } else {
            None
        }
    }

    // checks at two pixel increments along the line from
    // start to end for anything not allowed by filter
    // and if anything is found returns false
    // otherwise true, giving a good approximation
    // of a line of sight calculator
    // (this is a pretty slow way of doing this so
    // only use during level initialisation, on
    // the level being spawned in use rapier's method
    // instead as it will be much better optimised)
    pub fn line_of_sight<T>(
        &self,
        start: GridPos,
        end: GridPos,
        mut filter: T,
    ) -> bool
    where
        T: FnMut(&Tile) -> bool,
    {
        // getting the world positon of the start
        let mut pos = start.to_world();
        // getting the world position of the end
        let target = end.to_world();
        // getting the direction to travel from start to end
        let dir = (target - pos).normalize() * 2.0;
        // whilst the target hasn't been reached
        while !pos.abs_diff_eq(target, 6.0) {
            // step along the line between the
            // start and the end
            pos += dir;
            // if the tile at this point on the line
            // blocks the line of sight then
            // there it is blocked so return
            // early as false
            if !filter(&self[GridPos::from_world(pos)]) {
                return false;
            }
        }
        // if no point on the line has been found that
        // blocks the line of sight then there must be
        // a valid line of sight from start to end
        true
        // NOTE: this is quite an inneficient and error
        // prone methodology as it can skip over walls
        // and requires quite a lot of checks for any given
        // line.
        // however it works well enough and doesn't
        // have a noticable to the end user performance
        // impact so it's not worth wasting development
        // time on
    }

    // iterate over all the tiles in the grid
    pub fn iter(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter().flatten()
    }

    // iterate over all the tiles in the grid mutably
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut Tile> {
        self.tiles.iter_mut().flatten()
    }
}

// these allow indexing the Grid
// with GridPos rather than
// just raw indicies which increases safety as
// a GridPos cannot be created which refers to
// a tile that isn't on the grid
impl Index<GridPos> for Grid {
    type Output = Tile;
    fn index(&self, index: GridPos) -> &Self::Output {
        &self.tiles[index.y][index.x]
    }
}

// same as above but for mutating tiles
// by index on the Grid
impl IndexMut<GridPos> for Grid {
    fn index_mut(
        &mut self,
        index: GridPos,
    ) -> &mut Self::Output {
        &mut self.tiles[index.y][index.x]
    }
}

mod tile;
// exporting these so they can be imported from
// crate::grid rather than crate::grid::tile
pub use tile::{Tile, TileBundle, TileContent, TileSpawn};

mod pos;
pub use pos::GridPos;

mod generate;

// provide a boundary around the edge
// of the level to prevent physics objects going offscreen
pub struct Walls;

impl Walls {
    pub fn spawn(mut commands: Commands) {
        commands.spawn_bundle(ColliderBundle {
            // creating a polyline colldier around
            // the edge of the level
            shape: ColliderShape::polyline(
                // the verticies of the
                // rectangle around the edge of the screen
                vec![
                    Point2::from_slice(&[
                        -crate::WINDOW_WIDTH / 2.0,
                        crate::WINDOW_HEIGHT / 2.0,
                    ]),
                    Point2::from_slice(&[
                        crate::WINDOW_WIDTH / 2.0,
                        crate::WINDOW_HEIGHT / 2.0,
                    ]),
                    Point2::from_slice(&[
                        -crate::WINDOW_WIDTH / 2.0,
                        -crate::WINDOW_HEIGHT / 2.0,
                    ]),
                    Point2::from_slice(&[
                        crate::WINDOW_WIDTH / 2.0,
                        -crate::WINDOW_HEIGHT / 2.0,
                    ]),
                ],
                // indicating which order the indexes
                // should be read in to form a rectangle
                // (like the edges between each vertex)
                Some(vec![[0, 1], [1, 3], [3, 2], [2, 0]]),
            ),
            flags: ColliderFlags {
                collision_groups: phys::masks::wall(),
                ..Default::default()
            },
            ..Default::default()
        });
    }
}

pub mod difficulty;
pub use difficulty::Difficulty;
