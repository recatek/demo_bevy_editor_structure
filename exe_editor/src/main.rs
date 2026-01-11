use std::fs;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use libloading;
use toml::Table;

const PATH_TO_BEVY_TOML: &str = "bevy/bevy.toml";

#[derive(Resource, Default)]
struct ProjectData {
    toml_table: Option<Table>,
    files: Vec<PathBuf>,
}

#[derive(Resource, Default)]
struct GameLibrary {
    library: Option<libloading::Library>,
    registry: Option<TypeRegistry>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            file_path: "../bevy".into(),
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

fn show_reflected_data(
    mut contexts: EguiContexts,
    mut library: ResMut<GameLibrary>,
    registry: Res<AppTypeRegistry>,
) -> Result {
    let game_struct_info = get_game_struct_info(&*library, &*registry);

    egui::Window::new("Reflected Data").show(contexts.ctx_mut()?, |ui| {
        ui.vertical(|ui| {
            if library.library.is_some() {
                if ui.button("Unload Lib").clicked() {
                    library.unload_lib();
                }
            } else {
                if ui.button("Load Lib").clicked() {
                    library.load_lib();
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

/// Removes the bevy/ from the front of the path.
fn trim_path(path: &Path) -> PathBuf {
    path.components().skip(1).collect()
}

#[allow(unused_mut)]
#[allow(unused_variables)]
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

        unsafe {
            let mut registry = TypeRegistry::empty();
            let library =
                libloading::Library::new("lib_game.dll").expect("failed to load game dll");
            let register = library
                .get::<fn(&mut TypeRegistry)>(b"do_registration\0")
                .expect("failed to load symbol -- is the game dylib up to date?");
            register(&mut registry);

            self.library = Some(library);
            self.registry = Some(registry);
        }
    }
}
