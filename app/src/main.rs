use bevy::prelude::*;

mod scenes;
mod colors;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin { ..default() })
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
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
