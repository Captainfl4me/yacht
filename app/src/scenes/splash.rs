//! This plugin will display a splash screen with Bevy logo for n second before switching to the menu
use super::{despawn_screen, AppState};
use crate::style;
use bevy::prelude::*;

#[derive(Component)]
struct OnSplashScreen;

#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

pub fn splash_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Splash), splash_setup)
        .add_systems(Update, countdown.run_if(in_state(AppState::Splash)))
        .add_systems(OnExit(AppState::Splash), despawn_screen::<OnSplashScreen>);
}

fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon = asset_server.load("icon.png");
    commands
        .spawn((style::screen_root(), OnSplashScreen))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![
                        (
                            ImageNode::new(icon),
                            Node {
                                width: Val::Px(180.0),
                                margin: UiRect::bottom(Val::Px(24.0)),
                                ..default()
                            },
                        ),
                        style::label("YATCH", 40.0),
                        style::label_dim("a dice game with friends", 14.0),
                    ],
                ));
        });

    commands.insert_resource(SplashTimer(Timer::from_seconds(2.0, TimerMode::Once)));
}

fn countdown(
    mut game_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    if timer.tick(time.delta()).finished() {
        game_state.set(AppState::Menu);
    }
}
