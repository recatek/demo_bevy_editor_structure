use std::fs;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use libloading;
use toml::Table;

const PATH_TO_BEVY_TOML: &str = "assets/bevy.toml";

/// Contains information about the assets in the assets directory, including basic configuration
/// information (project name, etc.) stored in bevy.toml. Also caches the discovered files.
#[derive(Resource, Default)]
struct ProjectData {
    toml_table: Option<Table>,
    files: Vec<PathBuf>,
}

/// This represents the loaded dylib data and the extracted TypeRegistry from it.
#[derive(Resource, Default)]
struct GameLibrary {
    /// Pointer to the lib_game dylib when it's loaded, used for extracting type info.
    library: Option<libloading::Library>,
    /// We should keep a separate TypeRegistry from the global type registry because this
    /// data will periodically be cleared out and reloaded if the dylib is unloaded/reloaded.
    registry: Option<TypeRegistry>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            // Bevy doesn't seem to like workspace-level asset directories, preferring
            // crate-level asset directories. This bumps it up one level to the workspace.
            file_path: "../assets".into(),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(ProjectData::default())
        .insert_resource(GameLibrary::default())
        .add_systems(Startup, (setup_camera_system, setup_project_data))
        .add_systems(
            EguiPrimaryContextPass,
            (show_reflected_data, show_files_in_project),
        )
        .run();
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Loads asset information from the assets/ directory, along with the bevy.toml config data.
fn setup_project_data(mut project: ResMut<ProjectData>) {
    let contents = fs::read_to_string(PATH_TO_BEVY_TOML).unwrap();
    let toml_table: Table = contents.parse().unwrap();
    project.toml_table = Some(toml_table);

    let root_path = PathBuf::from(PATH_TO_BEVY_TOML)
        .parent()
        .unwrap()
        .to_path_buf();
    walk_files(&root_path, &mut project.files).unwrap();
}

/// Shows the loaded reflection type information, if any, from the loaded lib_game dylib.
fn show_reflected_data(
    mut contexts: EguiContexts,
    mut library: ResMut<GameLibrary>,
    registry: Res<AppTypeRegistry>,
) -> Result {
    let game_struct_info = get_game_struct_info(&*library, &*registry);

    egui::Window::new("Reflected Data").show(contexts.ctx_mut()?, |ui| {
        ui.vertical(|ui| {
            match library.library {
                Some(_) => {
                    if ui.button("Unload Lib").clicked() {
                        library.unload_lib();
                    }
                }
                None => {
                    if ui.button("Load Lib").clicked() {
                        library.load_lib();
                    }
                }
            }

            for (struct_name, fields) in game_struct_info {
                ui.label(struct_name);

                for field in fields {
                    ui.label(format!(" - {}", field));
                }
            }
        })
    });

    Ok(())
}

/// Displays UI for the project data, both the bevy.toml data and the contained asset paths.
fn show_files_in_project(mut contexts: EguiContexts, project: Res<ProjectData>) -> Result {
    egui::Window::new("Project Data").show(contexts.ctx_mut()?, |ui| {
        let table = project.toml_table.as_ref().unwrap();
        ui.label(format!(
            "Project Name: {}",
            table["bevy"]["name"].as_str().unwrap()
        ));

        ui.separator();

        ui.label("Files:");

        for path in project.files.iter() {
            ui.label(format!("{}", path.display()));
        }
    });
    Ok(())
}

/// Walks all the files in the given directory.
fn walk_files(root: &PathBuf, files: &mut Vec<PathBuf>) -> IoResult<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        match path.is_dir() {
            true => walk_files(&path, files)?,
            false => files.push(trim_path(&path)),
        }
    }
    Ok(())
}

/// Removes the assets/ from the front of the path.
fn trim_path(path: &Path) -> PathBuf {
    path.components().skip(1).collect()
}

/// Walks the loaded type registry data and displays all the lib_game structs and their fields.
fn get_game_struct_info(
    library: &GameLibrary,
    registry: &AppTypeRegistry,
) -> Vec<(String, Vec<String>)> {
    let mut result = Vec::new();
    let registry_lock = registry.internal.read().unwrap();
    let mut registry = &*registry_lock;

    if let Some(library_registry) = library.registry.as_ref() {
        registry = library_registry;
    }

    for data in registry.iter() {
        let type_info = data.type_info();
        if type_info.type_path().starts_with("lib_game") {
            if let Ok(struct_info) = type_info.as_struct() {
                let mut fields = Vec::new();

                for field in struct_info.iter() {
                    fields.push(field.name().to_string());
                }

                result.push((type_info.type_path().to_string(), fields));
            }
        }
    }

    result
}

impl GameLibrary {
    fn unload_lib(&mut self) {
        // Must do this before unloading the library to avoid a segfault (ask me how I know)
        drop(self.registry.take());

        if let Some(library) = self.library.take() {
            match library.close() {
                Ok(_) => {}
                Err(e) => error!("{e}"),
            }
        }
    }

    fn load_lib(&mut self) {
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
                libloading::Library::new("lib_game_outer.dll").expect("failed to load game dylib");
            let register = library
                .get::<fn(&mut TypeRegistry)>(b"do_registration\0")
                .expect("failed to load symbol -- is the game dylib up to date?");
            register(&mut registry);

            self.library = Some(library);
            self.registry = Some(registry);
        }
    }
}
