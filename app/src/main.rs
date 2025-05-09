use bevy::{log::LogPlugin, prelude::*};
use bevy_persistent::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

mod colors;
mod network;
mod scenes;

#[derive(Resource, Serialize, Deserialize)]
struct PlayerData {
    uuid: Uuid,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin { ..default() })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_plugins(scenes::scenes_plugin)
        .add_plugins(network::network_plugin)
        .add_plugins(TextInputPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let config_dir = dirs::data_local_dir()
        .unwrap_or(Path::new("local").join("data"))
        .join("yatch");
    commands.insert_resource(
        Persistent::<PlayerData>::builder()
            .name("Player data")
            .format(StorageFormat::TomlPretty)
            .path(config_dir.join("player_data.toml"))
            .default(PlayerData {
                uuid: Uuid::new_v4(),
            })
            .build()
            .expect("failed to initialize PlayerData"),
    )
}
