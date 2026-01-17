use std::thread;

use bevy::prelude::*;

use bevy::app::AppLabel;
use bevy::app::MainScheduleOrder;
use bevy::ecs::message::MessageRegistry;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;

use crate::GameLibrary;

#[derive(Clone, Hash, Debug, PartialEq, Eq, AppLabel)]
pub struct SubAppLabel;

pub fn build_sub_app(library: &GameLibrary) -> SubApp {
    let apply = library.get_apply().unwrap();

    let mut sub_app = SubApp::new();

    sub_app
        .insert_resource(AppTypeRegistry::default())
        .insert_resource(MessageRegistry::default())
        .insert_resource(MainScheduleOrder::default())
        .add_plugins(
            DefaultPlugins
                .build()
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
        );

    sub_app
}

pub fn build_thread_app(library: &GameLibrary) {
    thread::spawn(|| {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_systems(Update, game_system)
            .run();

        println!("finished?");
    });
}

fn game_system(mut count: Local<u32>) {
    println!("game system! {}", *count);
    *count = *count + 1;
}
