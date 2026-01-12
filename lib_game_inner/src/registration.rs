use bevy::reflect::TypeRegistry;

/// We use the inventory crate to gather all of the editor-facing reflected types.
/// This gives us "magic" discovery of all the types that use the inventory::submit macro.
pub struct EditorRegistrar(pub fn(&mut TypeRegistry));
inventory::collect!(EditorRegistrar);

/// Registers all of the inventory-submit!-aware types with the given TypeRegistry.
#[unsafe(no_mangle)]
pub fn do_registration(registry: &mut TypeRegistry) {
    for registrar in inventory::iter::<EditorRegistrar>() {
        registrar.0(registry)
    }
}
