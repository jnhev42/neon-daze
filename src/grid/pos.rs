use crate::grid;
use bevy::prelude::*;
use rand::{rngs::ThreadRng, Rng};
use std::{
    convert::TryInto,
    ops::{Add, Sub},
};

// stores positions on the grid
// garunteed to never be out of
// the grid boundaries
#[derive(
    Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash,
)]
pub struct GridPos {
    pub x: usize,
    pub y: usize,
}

impl GridPos {
    // furthest position on the grid
    // (bottom right)
    pub const MAX: Self = Self {
        x: grid::Grid::WIDTH - 1,
        y: grid::Grid::HEIGHT - 1,
        // -1 because vectors are 0
        // indexed
    };
    // origin position on the grid
    // (top left)
    pub const MIN: Self = Self { x: 0, y: 0 };

    // tries to create a new grid
    // position but will return
    // errors if over or under
    // T indicates it will work with any number
    // that can be cast into a unsigned interger
    pub fn try_new<T>(x: T, y: T) -> Option<Self>
    where
        T: TryInto<usize>,
    {
        macro_rules! try_conv {
            ($num:ident =< $max:expr) => {
                match $num.try_into() {
                    // failed to fit into a memory
                    // index so no new GridPos created
                    Err(_) => return None,
                    // is bigger than the specifed max
                    // index so fail creation
                    Ok(n) if n > $max => return None,
                    // otherwise its valid so
                    // converted form
                    Ok(n) => n,
                }
            };
        }
        Some(GridPos {
            x: try_conv!(x =< GridPos::MAX.x),
            y: try_conv!(y =< GridPos::MAX.y),
        })
    }
    // creates a new GridPos and if the indicies
    // passed are out of the grid bounds just rounds
    // to the closest edge
    pub fn new_bounded<T>(x: T, y: T) -> Self
    where
        T: TryInto<usize> + PartialOrd,
    {
        // using a macro to extend the syntax
        // this will wrap a number to be less
        // than the max that's passed and also
        // greater than zero
        // using a macro will improve performance
        // as it is expanded at compile time
        macro_rules! clamp {
            ($num:ident =< $max:expr) => {
                match $num.try_into() {
                    // if it failed to cast into a
                    // unsigned interger then
                    // it was probably negative
                    // so clamp to closest val
                    // which is zero
                    // (note that if really
                    // large indexes are passed
                    // then they will also cast
                    // to zero)
                    Err(_) => 0,
                    // if greater than the max
                    // return the max value
                    Ok(n) if n > $max => $max,
                    // otherwise its within range
                    // so just return the value
                    Ok(n) => n,
                }
            };
        }
        GridPos {
            x: clamp!(x =< Self::MAX.x),
            y: clamp!(y =< Self::MAX.y),
        }
    }

    // conveinece function for creating
    // GridPos when sure that its with valid
    pub fn new<T>(x: T, y: T) -> Self
    where
        T: TryInto<usize>,
    {
        Self::try_new(x, y).unwrap()
    }

    // iterates over every position in the grid
    pub fn iter_all() -> impl Iterator<Item = GridPos> {
        (0..grid::Grid::HEIGHT)
            .map(|y| {
                (0..grid::Grid::WIDTH)
                    .map(move |x| GridPos { x, y })
            })
            .flatten()
        // interestingly despite looking really
        // inefficient this actually runs pretty well
    }

    // converts a grid position to a
    // game world coordinate
    pub fn to_world(&self) -> Vec2 {
        Vec2::new(
            // x + 0.5 gets the center of the square
            (self.x as f32 + 0.5)
                // multiplying by the width of a tile
                * (crate::WINDOW_WIDTH / grid::Grid::WIDTH as f32)
                // taking away half the screen res as the
                // grids origin is top-left but the world
                // coords origin is the center of the screen
                - (crate::WINDOW_WIDTH / 2.0),
            // same reasons as above
            (self.y as f32 + 0.5)
                * (crate::WINDOW_WIDTH
                    / grid::Grid::HEIGHT as f32)
                - (crate::WINDOW_HEIGHT / 2.0),
        )
    }

    // takes a vector and returns the grid
    // positon of that vector
    pub fn from_world(pos: Vec2) -> GridPos {
        // converts v, a vector dimension, into a
        // usize given the dimension of the total
        // window and the width of the total grid
        macro_rules! to_grid {
            ($v:expr, $wndw:expr, $grid: expr) => {
                ((($v + ($wndw / 2.0)) / $wndw)
                    * $grid as f32)
                    .round() as usize
            };
        }
        GridPos::try_new(
            to_grid!(
                pos.x,
                crate::WINDOW_WIDTH,
                grid::Grid::WIDTH
            ),
            to_grid!(
                pos.y,
                crate::WINDOW_HEIGHT,
                grid::Grid::HEIGHT
            ),
        )
        .unwrap_or_else(|| GridPos::new(0, 0))
    }

    // generates a new random grid position
    // inside the specfied range
    pub fn random(
        rng: &mut ThreadRng,
        min: GridPos,
        max: GridPos,
    ) -> Self {
        Self {
            x: rng.gen_range(min.x..=max.x),
            y: rng.gen_range(min.y..=max.y),
        }
    }
}

// allows adding GridPositions together
impl Add for GridPos {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        GridPos::new(self.x + rhs.x, self.y + rhs.y)
    }
}

// allows subtracting grid positions from each other
impl Sub for GridPos {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        GridPos::new(self.x - rhs.x, self.y - rhs.y)
    }
}

// allows adding an index to grid positions
impl Add<usize> for GridPos {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        GridPos::new(self.x + rhs, self.y + rhs)
    }
}

// allows subtracting an index from grid positions
impl Sub<usize> for GridPos {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        GridPos::new(self.x - rhs, self.y - rhs)
    }
}
