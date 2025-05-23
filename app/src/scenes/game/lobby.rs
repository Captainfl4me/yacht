use bevy::prelude::*;
use bevy_persistent::Persistent;
use shared::{PlayerHeader, RoomInfoCommand, ServerAPICommand};
use uuid::Uuid;

use crate::{
    colors::{BACKGROUND_COLOR, NORMAL_BUTTON, TEXT_COLOR},
    network::{
        events::{RoomInfoEvent, RoomUpdateEvent},
        NetworkCommandEvent,
    },
    PlayerData,
};

use super::MenuButtonAction;

#[derive(Component)]
pub struct TopLevelLobbyScreen;

#[derive(Component)]
pub struct LobbyPanel;

#[derive(Component)]
pub struct PlayersList;

#[derive(Component)]
pub struct PlayerUuid(pub Uuid);

pub fn lobby_setup(
    mut commands: Commands,
    mut network_command_event: EventWriter<NetworkCommandEvent>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            TopLevelLobbyScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                BackgroundColor(BACKGROUND_COLOR),
                LobbyPanel,
            ));
        });

    network_command_event.write(NetworkCommandEvent(ServerAPICommand::RoomInfo(
        RoomInfoCommand,
    )));
}

pub fn join_room_update(
    mut commands: Commands,
    mut list_room_event: EventReader<RoomInfoEvent>,
    query_panel: Query<Entity, With<LobbyPanel>>,
    player_data: Res<Persistent<PlayerData>>,
) {
    if let Some(RoomInfoEvent(room_info)) = list_room_event.read().next() {
        if let Some(lobby_panel_node) = query_panel.iter().next() {
            commands.entity(lobby_panel_node).with_children(|parent| {
                parent.spawn((
                    Text::new(room_info.0.name.clone()),
                    TextFont {
                        font_size: 67.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                ));

                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            ..default()
                        },
                        PlayersList,
                    ))
                    .with_children(|parent| {
                        for player_header in room_info.0.players.iter() {
                            parent.spawn(player_item(player_header.clone()));
                        }
                    });

                if room_info.0.creator == player_data.uuid {
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
                            MenuButtonAction::StartGame,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Start game"),
                                TextFont {
                                    font_size: 33.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                }
            });
        }
    }
}

fn player_item(player_header: PlayerHeader) -> impl Bundle + use<> {
    (
        Node {
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
        PlayerUuid(player_header.uuid),
        children![(
            Text::new(player_header.name.clone()),
            TextFont {
                font_size: 33.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        )],
    )
}

pub fn player_list_update(
    mut commands: Commands,
    mut room_update_event: EventReader<RoomUpdateEvent>,
    query_player_list: Query<Entity, With<PlayersList>>,
    query_players_item: Query<(Entity, &PlayerUuid)>,
) {
    if let Some(RoomUpdateEvent(room_update_reason)) = room_update_event.read().next() {
        match room_update_reason.0.clone() {
            shared::RoomUpdateReason::NewPlayer(new_player) => {
                if let Some(player_list_node) = query_player_list.iter().next() {
                    info!("Player join {}", new_player.uuid);
                    commands.entity(player_list_node).with_children(|parent| {
                        parent.spawn(player_item(new_player));
                    });
                }
            }
            shared::RoomUpdateReason::PlayerLeft(player_uuid) => {
                for (entity_item, PlayerUuid(item_uuid)) in query_players_item {
                    if *item_uuid == player_uuid {
                        commands.entity(entity_item).despawn();
                    }
                }
            }
            _ => (),
        }
    }
}
