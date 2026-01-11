use bevy::prelude::*;

use lib_game::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            file_path: "../bevy".into(),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}
