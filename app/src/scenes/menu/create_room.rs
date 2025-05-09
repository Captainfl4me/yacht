use super::{MenuButtonAction, SubMenuScreen, SubSceneParentNode};
use crate::{
    colors::{NORMAL_BUTTON, TEXT_COLOR},
    network::NetworkManager,
    scenes::GameState,
};
use bevy::prelude::*;
use bevy_simple_text_input::{
    TextInput, TextInputPlaceholder, TextInputSettings, TextInputTextFont,
};
use shared::{CreateRoomCommand, CreateRoomResponse, ServerAPICommand, ServerAPIResponse};

#[derive(Resource)]
pub struct CreatingRoomName(pub String);

pub fn create_room_menu_setup(
    mut commands: Commands,
    query: Query<Entity, With<SubSceneParentNode>>,
) {
    if let Some(sub_scene_node) = query.iter().next() {
        commands.entity(sub_scene_node).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    SubMenuScreen,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextInput,
                        TextInputPlaceholder {
                            value: "Room name".to_string(),
                            ..default()
                        },
                        TextInputTextFont(TextFont {
                            font_size: 33.0,
                            ..default()
                        }),
                        TextInputSettings {
                            retain_on_submit: true,
                            ..default()
                        },
                        Node {
                            padding: UiRect::all(Val::Px(5.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            margin: UiRect::all(Val::Px(20.0)),
                            height: Val::Px(65.0),
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                    ));

                    parent
                        .spawn((
                            Button,
                            Node {
                                height: Val::Px(65.0),
                                margin: UiRect::all(Val::Px(20.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::CreateRoom,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("create"),
                                TextFont {
                                    font_size: 33.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });

                    parent
                        .spawn((
                            Button,
                            Node {
                                height: Val::Px(65.0),
                                margin: UiRect::all(Val::Px(20.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::BackToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("back"),
                                TextFont {
                                    font_size: 33.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                });
        });
    } else {
        error!("Subscene setup occurs before Menu setup!");
    }
}

pub fn creating_room_setup(
    mut commands: Commands,
    mut nm: ResMut<NetworkManager>,
    query: Query<Entity, With<SubSceneParentNode>>,
    room_name: Res<CreatingRoomName>,
) {
    if let Some(sub_scene_node) = query.iter().next() {
        info!("create room: {}", room_name.0);
        nm.send_queue
            .push_back(ServerAPICommand::CreateRoom(CreateRoomCommand(
                room_name.0.clone(),
            )));

        commands.entity(sub_scene_node).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        height: Val::Px(65.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    SubMenuScreen,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Creating..."),
                        TextFont {
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                });
        });
    }
}

pub fn creating_room_update(
    mut game_state: ResMut<NextState<GameState>>,
    mut nm: ResMut<NetworkManager>,
) {
    if let Some(ServerAPIResponse::CreateRoom(CreateRoomResponse(room_uuid))) =
        nm.read_queue.front()
    {
        nm.read_queue.pop_front();
        game_state.set(GameState::Game);
    }
}
