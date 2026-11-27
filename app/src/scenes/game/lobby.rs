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
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                BackgroundColor(BACKGROUND_COLOR),
                LobbyPanel,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Button,
                    Node {
                        height: Val::Px(65.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    LobbyButtonAction::LeaveRoom,
                    children![(
                        Text::new("Leave room"),
                        TextFont {
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    )],
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

                parent.spawn((
                    Text::new(format!("Room code: {}", room_info.0.uuid)),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
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
                            LobbyButtonAction::StartGame,
                            StartGameButton,
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
                parent.spawn((
                    Button,
                    Node {
                        height: Val::Px(65.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    LobbyButtonAction::StartGame,
                    StartGameButton,
                    children![(
                        Text::new("Start game"),
                        TextFont {
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    )],
                ));
            });
        }
    } else if !i_am_creator {
        for entity in &query_start {
            commands.entity(entity).despawn();
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::NetworkCommandEvent;

    #[test]
    fn test_lobby_button_action_sends_network_commands() {
        let mut app = App::new();
        app.add_event::<NetworkCommandEvent>()
            .add_systems(Update, button_action);

        app.world_mut()
            .spawn((Button, LobbyButtonAction::StartGame, Interaction::Pressed));
        app.world_mut()
            .spawn((Button, LobbyButtonAction::LeaveRoom, Interaction::Pressed));
        app.update();

        let events = app.world().resource::<Events<NetworkCommandEvent>>();
        let mut cursor = events.get_cursor();
        let cmds: Vec<&ServerAPICommand> = cursor
            .read(events)
            .map(|NetworkCommandEvent(cmd)| cmd)
            .collect();
        assert!(cmds
            .iter()
            .any(|c| matches!(c, ServerAPICommand::StartGame(_))));
        assert!(cmds
            .iter()
            .any(|c| matches!(c, ServerAPICommand::LeaveRoom(_))));
    }

    #[test]
    fn test_player_list_update_adds_and_removes_players() {
        let mut app = App::new();
        app.add_event::<RoomUpdateEvent>().add_systems(Update, player_list_update);
        app.world_mut().spawn(PlayersList);

        let new_player = PlayerHeader {
            uuid: Uuid::new_v4(),
            name: "Bob".to_string(),
        };
        app.world_mut().send_event(RoomUpdateEvent(shared::RoomUpdateResponse(
            shared::RoomUpdateReason::NewPlayer(new_player.clone()),
        )));
        app.update();

        let mut q = app.world_mut().query::<&PlayerUuid>();
        let listed: Vec<Uuid> = q.iter(app.world()).map(|p| p.0).collect();
        assert_eq!(listed, vec![new_player.uuid]);

        app.world_mut().send_event(RoomUpdateEvent(shared::RoomUpdateResponse(
            shared::RoomUpdateReason::PlayerLeft(new_player.uuid),
        )));
        app.update();
        let mut q = app.world_mut().query::<&PlayerUuid>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }
}
