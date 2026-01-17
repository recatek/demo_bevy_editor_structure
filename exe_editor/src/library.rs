use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

use libloading::Library;

/// This represents the loaded dylib data and the extracted TypeRegistry from it.
#[derive(Resource, Default)]
pub struct GameLibrary {
    /// Pointer to the lib_game dylib when it's loaded, used for extracting type info.
    library: Option<Library>,
    /// We should keep a separate TypeRegistry from the global type registry because this
    /// data will periodically be cleared out and reloaded if the dylib is unloaded/reloaded.
    registry: Option<TypeRegistry>,
}

impl GameLibrary {
    pub fn is_loaded(&self) -> bool {
        self.library.is_some()
    }

    pub fn registry(&self) -> Option<&TypeRegistry> {
        self.registry.as_ref()
    }

    pub fn get_apply(&self) -> Option<fn(&mut App)> {
        self.library.as_ref().map(|library| unsafe {
            *library
                .get::<fn(&mut App)>(b"do_apply_to_app\0")
                .expect("failed to load symbol -- is the game dylib up to date?")
        })
    }

    pub fn unload_lib(&mut self) {
        // Must do this before unloading the library to avoid a segfault (ask me how I know)
        drop(self.registry.take());

        if let Some(library) = self.library.take() {
            match library.close() {
                Ok(_) => {}
                Err(e) => error!("{e}"),
            }
        }
    }

    pub fn load_lib(&mut self) {
        if let Some(library) = self.library.take() {
            let _ = library.close();
        }

        // This builds an empty TypeRegistry and hands it to the lib_game dylib by invoking the
        // do_registration function. Over on the lib_game side, the do_registration function
        // iterates over all the submitted types (via the inventory crate) and has those types
        // add themselves to the given type registry. Passing the TypeRegistry from the editor
        // host executable to the lib_game dylib is ABI-sensitive, but should work as long as
        // both are using the same version of bevy[_reflect], and were both built with the same
        // Rust compiler on the same machine.
        unsafe {
            let mut registry = TypeRegistry::empty();
            let library =
                libloading::Library::new("lib_game.dll").expect("failed to load game dylib");
            let register = library
                .get::<fn(&mut TypeRegistry)>(b"do_registration\0")
                .expect("failed to load symbol -- is the game dylib up to date?");
            register(&mut registry);

            self.library = Some(library);
            self.registry = Some(registry);
        }
    }
}
