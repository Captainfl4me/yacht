use bevy::{
    input::common_conditions::input_toggle_active,
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_persistent::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use colors::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
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
                .set(LogPlugin {
                    level: Level::DEBUG,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, button_system)
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

#[derive(Component)]
pub struct ButtonDisable;

type ButtonSystemInteractionQuery<'a, 'b, 'c> = Query<
    'b,
    'c,
    (&'a Interaction, &'a mut BackgroundColor),
    (Changed<Interaction>, With<Button>, Without<ButtonDisable>),
>;

fn button_system(mut interaction_query: ButtonSystemInteractionQuery) {
    for (interaction, mut background_color) in &mut interaction_query {
        *background_color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}
