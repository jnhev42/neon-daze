use crate::state;
use bevy::prelude::*;
pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Config>()
            .init_resource::<ItemManager>()
            // applying the items at the same time as
            // generating the level
            .add_system(
                ItemManager::apply
                    .exclusive_system()
                    .with_run_criteria(State::<
                        state::GameState,
                    >::on_enter(
                        state::GameState::LoadingLevel,
                    )),
            )
            .add_system(ItemManager::reset.system());
    }
}

// manages all the players items
pub struct ItemManager {
    item_ids: Vec<ItemId>,
}

// gives an empty player items list
impl Default for ItemManager {
    fn default() -> Self {
        Self { item_ids: vec![] }
    }
}

impl ItemManager {
    pub fn apply(world: &mut World) {
        // creating a new default config
        let mut config = Config::from_world(world);
        // generating a new store of items
        let mut flags = ConfigFlags::new();
        // fetching the list of TtemIds the player has
        let ItemManager { item_ids } =
            world.get_resource::<ItemManager>().unwrap();
        // adding every player item to the flags
        for id in item_ids.iter().cloned() {
            flags.add(id)
        }
        // converting all the ItemIds to Box<dyn Items>
        // so i can Item methods on them
        let items = flags
            .iter()
            .map(|(id, count)| (id.to_item(), *count))
            .collect::<Vec<_>>();
        // fancy macro that looks confusing but reduces code size
        // slightly
        // takes in two methods of item and calls the
        // first on every first occurence of a given item
        // and the second on every subsequent occurence
        macro_rules! apply {
            ($first:expr => $otherwise:expr) => {
                for (item, count) in items.iter() {
                    // calling the first method
                    $first(&**item, &mut config);
                    // repeating as many times as there
                    // are items - 1 as i have already
                    // called first
                    for _ in 0..count - 1 {
                        $otherwise(&**item, &mut config);
                    }
                }
            };
        }
        apply!(Item::add_first => Item::add);
        apply!(Item::mul_first => Item::mul);
        // doing misc the not fancy way as its a bit simpler
        for (item, _) in items.iter() {
            // since misc is only called once no need to
            // take into account the count of an item
            item.misc(&mut config, world, &flags)
        }
        // limiting the configs values in a range
        // so that no weird / buggy behaviour happens
        config.clamp();
        // setting configs flags to the flags
        // we've already computed
        config.flags = flags;
        // adding the config to the world
        // overwriting the old one
        world.insert_resource(config)
    }

    // resets the player's items when the game is over
    pub fn reset(
        mut items: ResMut<ItemManager>,
        mut events: EventReader<state::GameEvent>,
    ) {
        if events.iter().any(|event| {
            matches!(event, state::GameEvent::GameOver)
        }) {
            *items = ItemManager::default();
        }
    }

    pub fn list(&self) -> String {
        self.item_ids
            .iter()
            .map(|id| id.to_item().name())
            .collect::<Vec<_>>()
            .join(", ")
    }

    // gives the player this item
    pub fn add(&mut self, item_id: ItemId) {
        self.item_ids.push(item_id);
    }
}

mod config;
pub use config::*;

mod items;
pub use items::*;
