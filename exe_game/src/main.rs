use bevy::prelude::*;

use lib_game_outer::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            // Bevy doesn't seem to like workspace-level asset directories, preferring
            // crate-level asset directories. This bumps it up one level to the workspace.
            file_path: "../assets".into(),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}
