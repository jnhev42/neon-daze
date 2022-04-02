use bevy::prelude::*;

pub struct JustSpawnedPlugin;

impl Plugin for JustSpawnedPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // always running the show system
        app.add_system(JustSpawned::show.system());
    }
}

// attach to an entity
// that is currently set as not
// visible to make it visible
// next frame
pub struct JustSpawned;

impl JustSpawned {
    // runs every frame to show any entity
    // that is marked as just spawned
    fn show(
        mut commands: Commands,
        mut query: Query<
            (Entity, &mut Visible),
            With<JustSpawned>,
        >,
    ) {
        for (entity, mut visible) in query.iter_mut() {
            // remove the JustSpawned marker so that
            // it doesn't get changed again
            commands.entity(entity).remove::<JustSpawned>();
            // make the sprite visible
            visible.is_visible = true;
        }
    }
}
