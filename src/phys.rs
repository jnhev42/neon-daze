macro_rules! group {
    ($name:ident = $memberships:expr, $filter:expr) => {
        pub fn $name() -> InteractionGroups {
            InteractionGroups::new($memberships, $filter)
        }
    };
}

// this contains all the bitmasks that rapier
// uses to filter collision groups
// i'm hiding it away in here so i don't have
// to look at it because its weird and confusing
pub mod masks {
    use bevy_rapier2d::prelude::*;
    // rapier filters collisions using bitmasks such that
    // for two objects to collide an and operation on their
    // bits must contain at least one one
    // (masks between the memberships and filter of every
    // physics entity)
    const NONE: u32 = 0b0;
    const WALL: u32 = 0b1;
    const PLAYER: u32 = 0b10;
    const PLAYER_BULLET: u32 = 0b100;
    const ENEMY: u32 = 0b1000;
    const ENEMY_VISION: u32 = 0b10000;

    group!(player = PLAYER, WALL + ENEMY + ENEMY_VISION);
    group!(
        wall = WALL,
        PLAYER + PLAYER_BULLET + ENEMY + ENEMY_VISION
    );
    group!(none = NONE, NONE);
    group!(player_bullet = PLAYER_BULLET, WALL + ENEMY);
    group!(
        enemy = ENEMY,
        PLAYER_BULLET + PLAYER + WALL + ENEMY
    );
    group!(enemy_vision = ENEMY_VISION, WALL + PLAYER);
}
