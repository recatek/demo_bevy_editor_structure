use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

use lib_game_inner::EditorRegistrar;

#[derive(Component, Default, Reflect)]
pub struct GameComponentFromOuter {
    pub width: u32,
    pub height: u32,
    pub scale: f32,
}

// This impl and submit call would be macro-generated via some derive.
impl GameComponentFromOuter {
    pub fn register_with(registry: &mut TypeRegistry) {
        registry.register::<Self>();
    }
}
inventory::submit! {
    EditorRegistrar(GameComponentFromOuter::register_with)
}
