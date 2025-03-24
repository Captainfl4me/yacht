use super::{MenuButtonAction, SubMenuScreen, SubSceneParentNode};
use crate::colors::{NORMAL_BUTTON, TEXT_COLOR};
use crate::network::NetworkManager;
use bevy::prelude::*;
use shared::{ListRoomCommand, ServerAPICommand};

#[derive(Component)]
pub struct RoomListSubNode;

#[derive(Resource)]
pub struct WaitForRoomList;

#[derive(Resource)]
pub struct JoiningRoomUuid(pub uuid::Uuid);

pub fn join_room_menu_setup(
    mut commands: Commands,
    query: Query<Entity, With<SubSceneParentNode>>,
    mut nm: ResMut<NetworkManager>,
) {
    if let Some(sub_scene_node) = query.iter().next() {
        commands.entity(sub_scene_node).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        ..default()
                    },
                    SubMenuScreen,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        RoomListSubNode,
                    ));

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(300.0),
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

        nm.send_queue
            .push_back(ServerAPICommand::ListRoom(ListRoomCommand));

        commands.insert_resource(WaitForRoomList);
    } else {
        error!("Subscene setup occurs before Menu setup!");
    }
}

pub fn join_room_update(
    mut commands: Commands,
    mut nm: ResMut<NetworkManager>,
    query: Query<Entity, With<RoomListSubNode>>,
    wait_for_room_list: Option<Res<WaitForRoomList>>,
) {
    if wait_for_room_list.is_some() {
        if let Some(shared::ServerAPIResponse::ListRoom(shared::ListRoomResponse(list))) =
            nm.read_queue.front()
        {
            if let Some(sub_scene_node) = query.iter().next() {
                commands.remove_resource::<WaitForRoomList>();

                for entry in list {
                    commands.entity(sub_scene_node).with_children(|parent| {
                        parent
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Px(300.0),
                                    height: Val::Px(65.0),
                                    margin: UiRect::all(Val::Px(20.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(NORMAL_BUTTON),
                                MenuButtonAction::JoinRoom(entry.uuid),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(entry.name.clone()),
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

            nm.read_queue.pop_front();
        }
    }
}

pub fn joining_room_setup(
    mut commands: Commands,
    query: Query<Entity, With<SubSceneParentNode>>,
    uuid: Res<JoiningRoomUuid>,
) {
    if let Some(sub_scene_node) = query.iter().next() {
        info!("Join room: {}", uuid.0);

        commands.entity(sub_scene_node).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
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
                        Text::new("Joining..."),
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
