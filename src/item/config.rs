use super::ItemId;
use crate::{grid, player};
use bevy::prelude::*;
use std::collections::HashMap;

// this stores all the data that the items
// can modify and is then used throughout the
// level to enact those behaviours
#[derive(Debug)]
pub struct Config {
    pub flags: ConfigFlags,
    pub player: player::PlayerBuilder,
    pub gun: player::GunBuilder,
}

impl FromWorld for Config {
    // returns a default version of the config
    fn from_world(world: &mut World) -> Self {
        Self {
            flags: ConfigFlags::new(),
            player: player::PlayerBuilder::from_world(
                world,
            ),
            gun: player::GunBuilder::from_world(world),
        }
    }
}

impl Config {
    // limits all the config values to sensible
    // amounts to avoid weird behaviour
    pub fn clamp(&mut self) {
        // limits a list of values to a given minimum and maximum
        // the list is delimited by semicolons and
        // each item takes the form
        // thing_to_limit => min: thing_min, max: thing_max;
        // which massively reduces code bloat for my use case
        // since every field on Config needs to be limited
        macro_rules! clamp {
            ($($val:expr => min: $min:expr, max: $max:expr);*;) => {
                $(
                    $val = if $val < $min {
                        $min
                    } else if $val > $max {
                        $max
                    } else {
                        $val
                    };
                )*
            };
        }
        // limiting all the values for the player
        let player = &mut self.player;
        clamp!(
            player.speed =>
            min: 50., max: 400.;
            player.size =>
            min: Vec2::splat(5.0),
            max: Vec2::splat(
                crate::WINDOW_HEIGHT
                / grid::Grid::HEIGHT as f32
            );
        );
        // limiting all the values for the gun
        let gun = &mut self.gun;
        clamp!(
            // limiting the x and y separately so
            // that if the x is overflowed it doesn't
            // also mark the y as overflowed
            gun.size.x =>
            min: 3.0,
            max: self.player.size.x * 0.9;
            gun.size.y =>
            min: 4.0,
            max: self.player.size.y;
            gun.cooldown =>
            min: 0.1, max: 2.0;
            gun.deviation =>
            min: 0.0, max: 2.0;
            gun.lifetime =>
            min: 0.3, max: 100.0;
        );
    }
}

// this is a wrapper for a hashmap that tells you
// items are and aren't in the player's inventory
// and how many of them the player has
#[derive(Debug)]
pub struct ConfigFlags(HashMap<ItemId, u32>);

impl ConfigFlags {
    // creates a new empty ConfigFlags
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    // checks if the given item is in the players
    // inventory
    pub fn contains(&self, flag: &ItemId) -> bool {
        self.0.contains_key(flag)
    }

    // counts the number of a given item in the player's
    // inventory
    pub fn count<'a>(&'a self, flag: &ItemId) -> &'a u32 {
        match self.0.get(flag) {
            Some(count) => count,
            None => &0,
        }
    }

    // adds an item to the player's invetory
    pub fn add(&mut self, flag: ItemId) {
        match self.0.get_mut(&flag) {
            // if there is already an entry for
            // this item, increment the count of
            // this item
            Some(count) => *count += 1,
            // insert a new entry in the hashmap
            // and give it a value of one
            None => {
                self.0.insert(flag, 1);
            }
        }
    }

    // returns an iterator over all the Items
    // and their counts
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&ItemId, &u32)> {
        self.0.iter()
    }
}
