use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

use crate::registration::EditorRegistrar;

#[derive(Component, Default, Reflect)]
pub struct GameComponentFromInner {
    pub name: String,
    pub value: f32,
}

// This impl and submit call would be macro-generated via some derive.
impl GameComponentFromInner {
    pub fn register_with(registry: &mut TypeRegistry) {
        registry.register::<Self>();
    }
}
inventory::submit! {
    EditorRegistrar(GameComponentFromInner::register_with)
}
