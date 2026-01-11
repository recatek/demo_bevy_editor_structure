use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use lib_game::SomeGameComponent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            file_path: "../bevy".into(),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup_camera_system)
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        .run();
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn ui_example_system(mut contexts: EguiContexts) -> Result {
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
