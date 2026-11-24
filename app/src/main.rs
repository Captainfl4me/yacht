use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
#[cfg(feature = "inspector")]
use bevy::input::common_conditions::input_toggle_active;
#[cfg(feature = "inspector")]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_persistent::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use colors::{
    ACCENT, ACCENT_HOVER, ACCENT_PRESSED, BG_COLOR, DISABLED_BUTTON, HOVERED_BUTTON, NORMAL_BUTTON,
    PRESSED_BUTTON,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

mod colors;
mod network;
mod scenes;
mod style;

fn default_player_name() -> String {
    "Player".to_string()
}

#[derive(Resource, Serialize, Deserialize)]
struct PlayerData {
    uuid: Uuid,
    #[serde(default = "default_player_name")]
    name: String,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                level: Level::DEBUG,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#bevy-canvas".to_string()),
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
    );
    #[cfg(feature = "inspector")]
    {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        });
        app.add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        );
    }
    app.add_systems(Startup, setup)
        .add_systems(Update, button_system)
        .add_plugins(scenes::scenes_plugin)
        .add_plugins(network::network_plugin)
        .add_plugins(TextInputPlugin)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.insert_resource(ClearColor(BG_COLOR));
    let mut background = ImageNode::new(asset_server.load("background.png"));
    background.image_mode = NodeImageMode::Stretch;
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        background,
        ZIndex(-1),
    ));

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
                name: default_player_name(),
            })
            .build()
            .expect("failed to initialize PlayerData"),
    )
}

#[derive(Component)]
pub struct ButtonDisable;

/// Marks a button that should use the accent color scheme (primary actions).
#[derive(Component)]
pub struct PrimaryButton;

fn button_system(
    mut normal_query: Query<
        (&Interaction, &mut BackgroundColor),
        (With<Button>, Without<ButtonDisable>, Without<PrimaryButton>),
    >,
    mut primary_query: Query<
        (&Interaction, &mut BackgroundColor),
        (With<Button>, Without<ButtonDisable>, With<PrimaryButton>),
    >,
    mut disabled_query: Query<&mut BackgroundColor, (With<Button>, With<ButtonDisable>)>,
) {
    for (interaction, mut background_color) in &mut normal_query {
        *background_color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        };
    }
    for (interaction, mut background_color) in &mut primary_query {
        *background_color = match *interaction {
            Interaction::Pressed => ACCENT_PRESSED.into(),
            Interaction::Hovered => ACCENT_HOVER.into(),
            Interaction::None => ACCENT.into(),
        };
    }
    for mut background_color in &mut disabled_query {
        *background_color = DISABLED_BUTTON.into();
    }
}
