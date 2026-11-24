use bevy::prelude::*;
use bevy_persistent::Persistent;
use shared::{PlayerHeader, RoomInfoCommand, ServerAPICommand};
use uuid::Uuid;

use crate::{
    colors::{BORDER_COLOR, NORMAL_BUTTON, PANEL_COLOR},
    network::{
        events::{RoomInfoEvent, RoomUpdateEvent},
        NetworkCommandEvent,
    },
    ui::{panel_node, pixel_text, pixel_title},
    PlayerData,
};

use super::RoomCreator;

#[derive(Component)]
pub enum LobbyButtonAction {
    StartGame,
    LeaveRoom,
}

#[derive(Component)]
pub struct StartGameButton;

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
            parent
                .spawn((
                    panel_node(600.0),
                    BackgroundColor(PANEL_COLOR),
                    BorderColor(BORDER_COLOR),
                    LobbyPanel,
                ))
                .with_children(|parent| {
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
                            LobbyButtonAction::LeaveRoom,
                            children![pixel_text("LEAVE ROOM", 18.0)],
                        ));
                });
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
                    pixel_title(room_info.0.name.clone(), 32.0),
                    Node {
                        align_self: AlignSelf::Center,
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));

                parent.spawn((
                    pixel_text(format!("ROOM CODE: {}", room_info.0.uuid), 11.0),
                    Node {
                        align_self: AlignSelf::Center,
                        margin: UiRect::bottom(Val::Px(16.0)),
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
                    parent.spawn(start_game_button());
                }
            });
        }
    }
}

fn start_game_button() -> impl Bundle {
    (
        Button,
        Node {
            height: Val::Px(56.0),
            margin: UiRect::all(Val::Px(12.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
        LobbyButtonAction::StartGame,
        StartGameButton,
        children![pixel_text("START GAME", 18.0)],
    )
}

fn player_item(player_header: PlayerHeader) -> impl Bundle + use<> {
    (
        Node {
            height: Val::Px(48.0),
            margin: UiRect::all(Val::Px(8.0)),
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
        BorderColor(BORDER_COLOR),
        PlayerUuid(player_header.uuid),
        children![pixel_text(player_header.name.clone(), 16.0)],
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

type LobbyActionInteractionQueryType<'a, 'b, 'c> =
    Query<'c, 'b, (&'a Interaction, &'a LobbyButtonAction), (Changed<Interaction>, With<Button>)>;

pub fn button_action(
    interaction_query: LobbyActionInteractionQueryType,
    mut network_command: EventWriter<NetworkCommandEvent>,
) {
    for (interaction, button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button_action {
                LobbyButtonAction::StartGame => {
                    network_command.write(NetworkCommandEvent(
                        shared::ServerAPICommand::StartGame(shared::StartGameCommand),
                    ));
                }
                LobbyButtonAction::LeaveRoom => {
                    network_command.write(NetworkCommandEvent(
                        shared::ServerAPICommand::LeaveRoom(shared::LeaveRoomCommand),
                    ));
                }
            }
        }
    }
}

pub fn creator_update(
    mut commands: Commands,
    room_creator: Res<RoomCreator>,
    player_data: Res<Persistent<PlayerData>>,
    query_panel: Query<Entity, With<LobbyPanel>>,
    query_start: Query<Entity, With<StartGameButton>>,
) {
    if !room_creator.is_changed() {
        return;
    }
    let i_am_creator = room_creator.0 == Some(player_data.uuid);
    if i_am_creator && query_start.is_empty() {
        if let Ok(panel) = query_panel.single() {
            commands.entity(panel).with_children(|parent| {
                parent.spawn(start_game_button());
            });
        }
    } else if !i_am_creator {
        for entity in &query_start {
            commands.entity(entity).despawn();
        }
    }
}
