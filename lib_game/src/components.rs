use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

use crate::registration::EditorRegistrar;

#[derive(Component, Default, Reflect)]
pub struct SomeGameComponent {
    pub name: String,
    pub value: f32,
}

#[derive(Component, Default, Reflect)]
pub struct AnotherGameComponent {
    pub width: u32,
    pub height: u32,
    pub scale: f32,
}

// This impl and submit call would be macro-generated via some derive.
impl SomeGameComponent {
    pub fn register_with(registry: &mut TypeRegistry) {
        registry.register::<Self>();
    }
}
inventory::submit! {
    EditorRegistrar(SomeGameComponent::register_with)
}

// This impl and submit call would be macro-generated via some derive.
impl AnotherGameComponent {
    pub fn register_with(registry: &mut TypeRegistry) {
        registry.register::<Self>();
    }
}
inventory::submit! {
    EditorRegistrar(AnotherGameComponent::register_with)
}
