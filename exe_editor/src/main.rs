use std::fs;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use toml::Table;

use lib_game::SomeGameComponent;

const PATH_TO_BEVY_TOML: &str = "bevy/bevy.toml";

#[derive(Resource, Default)]
struct ProjectData {
    toml_table: Option<Table>,
    files: Vec<PathBuf>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            file_path: "../bevy".into(),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(ProjectData::default())
        .add_systems(Startup, (setup_camera_system, read_project_root))
        .add_systems(
            EguiPrimaryContextPass,
            (show_reflected_data, show_files_in_project),
        )
        .run();
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn read_project_root(mut project: ResMut<ProjectData>) {
    let contents = fs::read_to_string(PATH_TO_BEVY_TOML).unwrap();
    let toml_table: Table = contents.parse().unwrap();
    project.toml_table = Some(toml_table);

    let root_path = PathBuf::from(PATH_TO_BEVY_TOML)
        .parent()
        .unwrap()
        .to_path_buf();
    walk_files(&root_path, &mut project.files).unwrap();
}

fn show_reflected_data(mut contexts: EguiContexts) -> Result {
    egui::Window::new("Reflected Data").show(contexts.ctx_mut()?, |ui| {
        let component = SomeGameComponent::default();
        if let Some(struct_info) = component.get_represented_struct_info() {
            for field_name in struct_info.field_names() {
                ui.label(*field_name);
            }
        }
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
    path.components().into_iter().skip(1).collect()
}
