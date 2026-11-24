use super::{super::AppState, MenuButtonAction, MenuState, SubMenuScreen, SubSceneParentNode};
use crate::{
    colors::NORMAL_BUTTON,
    network::{
        events::{JoinRoomEvent, ListRoomEvent},
        NetworkCommandEvent,
    },
    style,
};
use bevy::prelude::*;
use shared::{
    JoinRoomCommand, JoinRoomResponse, ListRoomCommand, ListRoomResponse, ServerAPICommand,
};

#[derive(Component)]
pub struct RoomListSubNode;

#[derive(Resource)]
pub struct WaitForRoomList;

#[derive(Resource)]
pub struct JoiningRoomUuid(pub uuid::Uuid);

pub fn join_room_menu_setup(
    mut commands: Commands,
    query: Query<Entity, With<SubSceneParentNode>>,
    mut network_command_event: EventWriter<NetworkCommandEvent>,
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
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            ..default()
                        },
                        RoomListSubNode,
                    ));

                    parent
                        .spawn((
                            Button,
                            style::button_node(),
                            style::button_border(),
                            style::radius(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::BackToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn(style::label("Back", 26.0));
                        });
                });
        });

        network_command_event.write(NetworkCommandEvent(ServerAPICommand::ListRoom(
            ListRoomCommand,
        )));

        commands.insert_resource(WaitForRoomList);
    } else {
        error!("Subscene setup occurs before Menu setup!");
    }
}

pub fn join_room_update(
    mut commands: Commands,
    mut list_room_event: EventReader<ListRoomEvent>,
    query: Query<Entity, With<RoomListSubNode>>,
    wait_for_room_list: Option<Res<WaitForRoomList>>,
) {
    if wait_for_room_list.is_some() {
        if let Some(ListRoomEvent(ListRoomResponse(list))) = list_room_event.read().next() {
            if let Some(sub_scene_node) = query.iter().next() {
                commands.remove_resource::<WaitForRoomList>();

                if list.is_empty() {
                    commands.entity(sub_scene_node).with_children(|parent| {
                        parent.spawn((
                            style::label_dim("No open rooms yet — create one!", 20.0),
                            Node {
                                margin: UiRect::vertical(Val::Px(12.0)),
                                align_self: AlignSelf::Center,
                                ..default()
                            },
                        ));
                    });
                }

                for entry in list {
                    commands.entity(sub_scene_node).with_children(|parent| {
                        parent
                            .spawn((
                                Button,
                                style::button_node(),
                                style::button_border(),
                                style::radius(),
                                BackgroundColor(NORMAL_BUTTON),
                                MenuButtonAction::JoinRoom(entry.uuid),
                            ))
                            .with_children(|parent| {
                                parent.spawn(style::label(entry.name.clone(), 24.0));
                            });
                    });
                }
            }
        }
    }
}

pub fn joining_room_setup(
    mut commands: Commands,
    mut network_command_event: EventWriter<NetworkCommandEvent>,
    query: Query<Entity, With<SubSceneParentNode>>,
    uuid: Res<JoiningRoomUuid>,
) {
    if let Some(sub_scene_node) = query.iter().next() {
        info!("Join room: {}", uuid.0);
        network_command_event.write(NetworkCommandEvent(ServerAPICommand::JoinRoom(
            JoinRoomCommand(uuid.0),
        )));

        commands.entity(sub_scene_node).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(16.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    SubMenuScreen,
                ))
                .with_children(|parent| {
                    parent.spawn(style::label_dim("Joining…", 26.0));
                });
        });
    }
}

pub fn joining_room_update(
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut join_room_event: EventReader<JoinRoomEvent>,
    uuid: Option<Res<JoiningRoomUuid>>,
) {
    if let Some(uuid) = uuid {
        if let Some(JoinRoomEvent(JoinRoomResponse(room_uuid))) = join_room_event.read().next() {
            if uuid.0 == *room_uuid {
                game_state.set(AppState::Game);
                commands.remove_resource::<JoiningRoomUuid>();
            } else {
                menu_state.set(MenuState::JoinRoom);
                error!("Room ID do not match");
            }
        }
    }
}
