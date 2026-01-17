mod library;
mod player;
mod winit_hack;

use std::fs;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use toml::Table;

use library::GameLibrary;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.build().set(AssetPlugin {
        // Bevy doesn't seem to like workspace-level asset directories, preferring
        // crate-level asset directories. This bumps it up one level to the workspace.
        file_path: "../assets".into(),
        ..default()
    }));
    app.add_systems(Startup, setup_scene);

    app.run();
}

fn setup_scene(mut commands: Commands) {
    let second_window = commands
        .spawn(Window {
            title: "Second window".to_owned(),
            ..default()
        })
        .id();
}

/*
const PATH_TO_BEVY_TOML: &str = "assets/bevy.toml";

static SHOULD_ADD_APP: AtomicBool = AtomicBool::new(false);

/// Contains information about the assets in the assets directory, including basic configuration
/// information (project name, etc.) stored in bevy.toml. Also caches the discovered files.
#[derive(Resource, Default)]
struct ProjectData {
    toml_table: Option<Table>,
    files: Vec<PathBuf>,
}

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                // Bevy doesn't seem to like workspace-level asset directories, preferring
                // crate-level asset directories. This bumps it up one level to the workspace.
                file_path: "../assets".into(),
                ..default()
            })
            .disable::<bevy::winit::WinitPlugin>(),
    );

    winit_hack::init_winit(&mut app, post_update);

    app.add_plugins(EguiPlugin::default())
        .insert_resource(ProjectData::default())
        .insert_resource(GameLibrary::default())
        .add_systems(Startup, (setup_camera_system, setup_project_data))
        .add_systems(
            EguiPrimaryContextPass,
            (show_reflected_data, show_files_in_project),
        )
        .run();
}

fn post_update(app: &mut App) {
    if let Some(library) = app.world().get_resource::<GameLibrary>() {
        if library.is_loaded() && SHOULD_ADD_APP.load(Ordering::Relaxed) {
            SHOULD_ADD_APP.store(false, Ordering::Relaxed);

            player::build_thread_app(library);
            //app.insert_sub_app(player::SubAppLabel, sub_app);
        }
    }
}

// fn app_runner(mut app: App) -> AppExit {
//     app.finish();
//     app.cleanup();

//     loop {
//         app.update();

//         if let Some(library) = app.world().get_resource::<GameLibrary>() {
//             if library.is_loaded() && SHOULD_ADD_APP.load(Ordering::Relaxed) {
//                 SHOULD_ADD_APP.store(false, Ordering::Relaxed);

//                 let sub_app = player::build_sub_app(library);
//                 app.insert_sub_app(player::SubAppLabel, sub_app);
//             }
//         }

//         if let Some(exit) = app.should_exit() {
//             return exit;
//         }
//     }
// }

// pub fn winit_runner(mut app: App, event_loop: EventLoop<WinitUserEvent>) -> AppExit {
//     if app.plugins_state() == PluginsState::Ready {
//         app.finish();
//         app.cleanup();
//     }

//     let runner_state = WinitAppRunnerState::new(app);

//     trace!("starting winit event loop");
//     // The winit docs mention using `spawn` instead of `run` on Wasm.
//     // https://docs.rs/winit/latest/winit/platform/web/trait.EventLoopExtWebSys.html#tymethod.spawn_app

//     let mut runner_state = runner_state;
//     if let Err(err) = event_loop.run_app(&mut runner_state) {
//         error!("winit event loop returned an error: {err}");
//     }
//     // If everything is working correctly then the event loop only exits after it's sent an exit code.
//     runner_state.app_exit.unwrap_or_else(|| {
//         error!("Failed to receive an app exit code! This is a bug");
//         AppExit::error()
//     })
// }

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
            match library.is_loaded() {
                true => {
                    if ui.button("Unload Lib").clicked() {
                        library.unload_lib();
                    }
                }
                false => {
                    if ui.button("Load Lib").clicked() {
                        library.load_lib();
                        SHOULD_ADD_APP.store(true, Ordering::Relaxed);
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

    if let Some(library_registry) = library.registry() {
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
*/
