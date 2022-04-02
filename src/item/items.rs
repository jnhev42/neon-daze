use rand::{prelude::ThreadRng, Rng};

use super::*;

// unique identifier for each item
// is useful as can't send Box<dyn Item>
// between threads (also can't save to file later)
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ItemId {
    AutoFire,
    Faster,
    Smaller,
    Bigger,
    Slower,
    Accuracy,
    Bouncy,
    HighCalibre,
    Laser,
}

impl ItemId {
    // converts an id to its item
    pub fn to_item(&self) -> Box<dyn Item> {
        // mimics a match but boxes the expression
        // that is matched to, making the syntax look less
        // janky
        macro_rules! match_boxed {
            ($($id:pat => $item:expr),*,) => {
                match self {
                    $(
                        $id => Box::new($item),
                    )*
                }
            }
        }
        // matching ids into items
        match_boxed!(
            ItemId::AutoFire => AutoFire,
            ItemId::Faster => Faster,
            ItemId::Smaller => Smaller,
            ItemId::Bigger => Bigger,
            ItemId::Slower => Slower,
            ItemId::Accuracy => Accuracy,
            ItemId::Bouncy => Bouncy,
            ItemId::HighCalibre => HighCalibre,
            ItemId::Laser => Laser,
        )
    }

    // generates a random item
    pub fn random(rng: &mut ThreadRng) -> ItemId {
        match rng.gen_range(0..8) {
            0 => ItemId::Accuracy,
            1 => ItemId::AutoFire,
            2 => ItemId::Bigger,
            3 => ItemId::Bouncy,
            4 => ItemId::Faster,
            5 => ItemId::HighCalibre,
            6 => ItemId::Laser,
            7 => ItemId::Slower,
            8 => ItemId::Smaller,
            _ => panic!("unreachable"),
        }
    }
}

// this is the trait that allows a struct
// to modify the items config
pub trait Item {
    // i have split item modifers up into
    // three main groups:
    // adding
    // subtracting
    // miscellaneous
    // adding and multiplying have been split to
    // prevent the order the items
    // are stored in from affecting the player's
    // stats
    // miscellaneous changes anything that isn't really
    // a directly important stat i.e. the color of the
    // player or something

    // run on the first occurence of this item
    // in the player's inventory
    // adding to their stats
    fn add_first(&self, config: &mut Config) {
        // default implementation of add first
        // just falls through to add
        self.add(config)
    }
    // run on any item after the first
    fn add(&self, _config: &mut Config) {}
    // run on the first occurence of this item
    // in the player's inventory
    // multiplying their stats
    fn mul_first(&self, config: &mut Config) {
        self.mul(config)
    }
    // run on any item after the first
    fn mul(&self, _config: &mut Config) {}

    // only run once per occurence of an item
    // applies miscelaneous effects like changing
    // the player's color that only make sense to apply
    // once
    fn misc(
        &self,
        _config: &mut Config,
        _world: &mut World,
        _flags: &ConfigFlags,
    ) {
    }
    // gives the id of the item for saving and loading it
    fn id(&self) -> ItemId;
    // gives the name of the item for ingame display
    fn name(&self) -> String;
    // gives the description of an item for ingame display
    fn desc(&self) -> String;
}

// shorthand for a method that returns the items
// id or name or desc depending
// since every item has to have this method so it
// reduces code bloat
macro_rules! id {
    ($id:expr) => {
        fn id(&self) -> ItemId {
            $id
        }
    };
}

macro_rules! name {
    ($name:literal) => {
        fn name(&self) -> String {
            $name.to_string()
        }
    };
}

macro_rules! desc {
    ($desc:literal) => {
        fn desc(&self) -> String {
            $desc.to_string()
        }
    };
}

// makes the players gun automatic
// so that they can hold down the
// trigger and it keeps shooting
// also makes the gun shoot faster
// and less accurate
pub struct AutoFire;

impl Item for AutoFire {
    fn add_first(&self, config: &mut Config) {
        config.gun.deviation += 0.3;
    }
    fn mul(&self, config: &mut Config) {
        config.gun.cooldown *= 0.9;
        config.gun.deviation *= 1.2;
    }
    id!(ItemId::AutoFire);
    name!("Automatic");
    desc!("Hit the broad side of a barn at 1000RPM");
}

// makes the player move faster
// but their bullets live less time
pub struct Faster;

impl Item for Faster {
    fn mul_first(&self, config: &mut Config) {
        config.player.speed *= 1.5;
        config.gun.lifetime *= 0.8;
    }
    fn mul(&self, config: &mut Config) {
        config.player.speed *= 1.3;
        config.gun.lifetime *= 0.8;
    }
    id!(ItemId::Faster);
    name!("Turbo the Snail");
    desc!("Go farther but shorten your bullets lives");
}

// makes player slower
// but bullets live longer

pub struct Slower;

impl Item for Slower {
    fn mul(&self, config: &mut Config) {
        config.player.speed *= 0.8;
        config.gun.lifetime *= 1.5;
    }
    id!(ItemId::Slower);
    name!("Lead Boots");
    desc!("No I only meant the tip");
}

// makes the player smaller
// but slower
pub struct Smaller;

impl Item for Smaller {
    fn add_first(&self, config: &mut Config) {
        config.player.size -= Vec2::splat(5.0);
    }
    fn mul(&self, config: &mut Config) {
        config.player.size *= 0.8;
        config.player.speed *= 0.8;
    }
    id!(ItemId::Smaller);
    // alice in wonderland reference
    name!("Drink Me");
    desc!("Shrinks and slows");
}

// makes the player bigger
// but also their bullets
// bigger
pub struct Bigger;

impl Item for Bigger {
    fn add_first(&self, config: &mut Config) {
        config.player.size += Vec2::splat(5.0);
    }
    fn mul(&self, config: &mut Config) {
        config.gun.size *= 1.2;
        config.player.size *= 1.1;
    }
    id!(ItemId::Bigger);
    name!("Magic Beans");
    desc!(
        "Fe Fi Fo Fum! I smell the blood of an Englishman"
    );
}

// makes you more accurate
// but shoot slower
pub struct Accuracy;

impl Item for Accuracy {
    fn mul(&self, config: &mut Config) {
        config.gun.cooldown *= 1.4;
        config.gun.deviation *= 0.6;
    }
    id!(ItemId::Accuracy);
    name!("Thinking Man");
    desc!("Calculate the likely trajectory of every shot you fire");
}

// shoots bouncing bullets
// they are less accurate
// and live less long
pub struct Bouncy;

impl Item for Bouncy {
    fn mul(&self, config: &mut Config) {
        config.gun.deviation *= 1.6;
        config.gun.lifetime *= 0.7;
    }
    fn misc(
        &self,
        config: &mut Config,
        _world: &mut World,
        _flags: &ConfigFlags,
    ) {
        config.gun.size = Vec2::new(3.0, 3.0)
            * (config.gun.size.length() / 10.0)
    }
    id!(ItemId::Bouncy);
    name!("Bouncy Bullets");
    desc!("Boing");
}

// makes the players bullets wider but also slower
// and they're more accurate
struct HighCalibre;

impl Item for HighCalibre {
    fn mul(&self, config: &mut Config) {
        config.gun.size.x *= 1.3;
        config.gun.size.y *= 0.9;
        config.gun.speed *= 0.8;
        config.gun.deviation *= 0.7;
    }
    id!(ItemId::HighCalibre);
    name!("Anti Tank Rounds");
    desc!("Excessive Force");
}

// makes the player's bullet longer
// and mover faster but gives it
// a longer cooldown
// also makes it red
struct Laser;

impl Item for Laser {
    fn mul(&self, config: &mut Config) {
        config.gun.speed *= 1.4;
        config.gun.cooldown *= 1.2;
        config.gun.size.x *= 0.9;
        config.gun.size.y *= 1.3;
    }
    fn misc(
        &self,
        config: &mut Config,
        world: &mut World,
        _flags: &ConfigFlags,
    ) {
        // making the gun and bullets red because laser
        let mut colors = world
            .get_resource_mut::<Assets<ColorMaterial>>()
            .unwrap();
        config.gun.material =
            colors.add(Color::rgb_u8(255, 0, 0).into());
    }
    id!(ItemId::Laser);
    name!("Laser");
    desc!("Shark not included");
}
