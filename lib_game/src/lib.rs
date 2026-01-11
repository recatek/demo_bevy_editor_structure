use bevy::prelude::*;

pub struct GamePlugin;

#[derive(Component, Default, Reflect)]
pub struct SomeGameComponent {
    pub name: String,
    pub value: f32,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(asset_server.load("sprites/icon.png")));
}
