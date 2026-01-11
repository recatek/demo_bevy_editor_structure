use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

pub struct GamePlugin;

pub struct EditorRegistrar(pub fn(&mut TypeRegistry));

inventory::collect!(EditorRegistrar);

#[derive(Component, Default, Reflect)]
pub struct SomeGameComponent {
    pub name: String,
    pub value: f32,
}

// This would be macro-generated
impl SomeGameComponent {
    pub fn register_with(registry: &mut TypeRegistry) {
        registry.register::<Self>();
    }
}
inventory::submit! {
    EditorRegistrar(SomeGameComponent::register_with)
}

#[derive(Component, Default, Reflect)]
pub struct SomeOtherGameComponent {
    pub width: u32,
    pub height: u32,
}

// This would be macro-generated
impl SomeOtherGameComponent {
    pub fn register_with(registry: &mut TypeRegistry) {
        registry.register::<Self>();
    }
}
inventory::submit! {
    EditorRegistrar(SomeOtherGameComponent::register_with)
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

#[unsafe(no_mangle)]
pub fn do_registration(registry: &mut TypeRegistry) {
    for registrar in inventory::iter::<EditorRegistrar>() {
        registrar.0(registry)
    }
}
