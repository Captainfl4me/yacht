use super::{MenuButtonAction, SubMenuScreen, SubSceneParentNode};
use crate::{
    colors::{BACKGROUND_COLOR, BORDER_COLOR, NORMAL_BUTTON},
    network::{events::CreateRoomEvent, NetworkCommandEvent},
    scenes::AppState,
    ui::pixel_text,
};
use bevy::prelude::*;
use bevy_simple_text_input::{
    TextInput, TextInputPlaceholder, TextInputSettings, TextInputTextFont,
};
use shared::{CreateRoomCommand, CreateRoomResponse, ServerAPICommand};

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
                            value: "ROOM NAME".to_string(),
                            ..default()
                        },
                        TextInputTextFont(TextFont {
                            font_size: 18.0,
                            ..default()
                        }),
                        TextInputSettings {
                            retain_on_submit: true,
                            ..default()
                        },
                        Node {
                            padding: UiRect::all(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(4.0)),
                            margin: UiRect::all(Val::Px(12.0)),
                            height: Val::Px(56.0),
                            ..default()
                        },
                        BackgroundColor(BACKGROUND_COLOR),
                        BorderColor(BORDER_COLOR),
                    ));

                    for (action, label) in [
                        (MenuButtonAction::CreateRoom, "CREATE"),
                        (MenuButtonAction::BackToMainMenu, "BACK"),
                    ] {
                        parent
                            .spawn((
                                Button,
                                Node {
                                    height: Val::Px(56.0),
                                    margin: UiRect::all(Val::Px(12.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(NORMAL_BUTTON),
                                action,
                            ))
                            .with_children(|parent| {
                                parent.spawn(pixel_text(label, 18.0));
                            });
                    }
                });
        });
    } else {
        error!("Subscene setup occurs before Menu setup!");
    }
}

pub fn creating_room_setup(
    mut commands: Commands,
    query: Query<Entity, With<SubSceneParentNode>>,
    mut network_command_event: EventWriter<NetworkCommandEvent>,
    room_name: Res<CreatingRoomName>,
) {
    if let Some(sub_scene_node) = query.iter().next() {
        info!("create room: {}", room_name.0);
        network_command_event.write(NetworkCommandEvent(ServerAPICommand::CreateRoom(
            CreateRoomCommand(room_name.0.clone()),
        )));

        commands.entity(sub_scene_node).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        height: Val::Px(56.0),
                        margin: UiRect::all(Val::Px(12.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    SubMenuScreen,
                ))
                .with_children(|parent| {
                    parent.spawn(pixel_text("CREATING...", 18.0));
                });
        });
    }
}

pub fn creating_room_update(
    mut game_state: ResMut<NextState<AppState>>,
    mut create_room_event: EventReader<CreateRoomEvent>,
) {
    if let Some(CreateRoomEvent(CreateRoomResponse(_))) = create_room_event.read().next() {
        game_state.set(AppState::Game);
    }
}
