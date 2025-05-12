use super::PlayerData;
use aeronet_io::{connection::Disconnected, Session};
use aeronet_websocket::client::{ClientConfig, WebSocketClient, WebSocketClientPlugin};
use bevy_persistent::Persistent;
use shared::{ServerAPICommand, ServerAPIResponse};
use speedy::{Readable, Writable};

use bevy::prelude::*;

pub mod events;
use events::{
    ChangeScoreEvent, ChangeTurnEvent, CreateRoomEvent, ErrorEvent, GameStateEvent, JoinRoomEvent,
    KeepDiceEvent, ListRoomEvent, RollEvent, RoomInfoEvent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum NetworkManagerState {
    #[default]
    Disconnect,
    Connecting,
    Connected,
}

#[derive(Event, Debug)]
pub struct NetworkCommandEvent(pub ServerAPICommand);

pub fn network_plugin(app: &mut App) {
    app.init_state::<NetworkManagerState>()
        .add_plugins(WebSocketClientPlugin)
        .add_systems(Startup, ws_setup)
        .add_systems(
            Update,
            (handle_websocket_response, handle_websocket_command)
                .run_if(not(in_state(NetworkManagerState::Disconnect))),
        )
        .add_observer(on_connected)
        .add_observer(on_disconnected)
        .add_event::<NetworkCommandEvent>()
        .add_event::<CreateRoomEvent>()
        .add_event::<ListRoomEvent>()
        .add_event::<JoinRoomEvent>()
        .add_event::<GameStateEvent>()
        .add_event::<RollEvent>()
        .add_event::<KeepDiceEvent>()
        .add_event::<ChangeTurnEvent>()
        .add_event::<ChangeScoreEvent>()
        .add_event::<ErrorEvent>()
        .add_event::<RoomInfoEvent>();
}

#[cfg(target_family = "wasm")]
fn client_config() -> ClientConfig {
    #[expect(
        clippy::default_constructed_unit_structs,
        reason = "keep parity with non-WASM"
    )]
    ClientConfig::default()
}

#[cfg(not(target_family = "wasm"))]
fn client_config() -> ClientConfig {
    ClientConfig::builder().with_no_cert_validation()
}

fn ws_setup(mut commands: Commands, mut nm_state: ResMut<NextState<NetworkManagerState>>) {
    let target = "ws://127.0.0.1:8080";
    let config = client_config();

    let name = format!("{}. {target}", 0);
    commands
        .spawn(Name::new(name))
        .queue(WebSocketClient::connect(config, target));

    nm_state.set(NetworkManagerState::Connecting);
}

fn on_connected(
    trigger: Trigger<OnAdd, Session>,
    names: Query<&Name>,
    mut nm_state: ResMut<NextState<NetworkManagerState>>,
    mut network_command_event: EventWriter<NetworkCommandEvent>,
    player_data: Res<Persistent<PlayerData>>,
) {
    let entity = trigger.target();
    let name = names
        .get(entity)
        .expect("our session entity should have a name");
    info!("{name} connected");

    nm_state.set(NetworkManagerState::Connected);
    network_command_event.write(NetworkCommandEvent(ServerAPICommand::Register(
        shared::RegisterCommand(player_data.uuid),
    )));
}

fn on_disconnected(
    trigger: Trigger<Disconnected>,
    names: Query<&Name>,
    mut nm_state: ResMut<NextState<NetworkManagerState>>,
) {
    let entity = trigger.target();
    let name = names
        .get(entity)
        .expect("our session entity should have a name");
    info!(
        "{name} disconnected: {}",
        match &*trigger {
            Disconnected::ByUser(reason) => {
                format!("by user: {reason}")
            }
            Disconnected::ByPeer(reason) => {
                format!("by peer: {reason}")
            }
            Disconnected::ByError(err) => {
                format!("due to error: {err:?}")
            }
        }
    );

    nm_state.set(NetworkManagerState::Disconnect);
}

fn handle_websocket_command(
    mut sessions: Query<(Entity, &Name, Option<&mut Session>)>,
    mut network_command_event: EventReader<NetworkCommandEvent>,
) {
    if let Ok((_, _, Some(mut ws_session))) = sessions.single_mut() {
        for NetworkCommandEvent(cmd) in network_command_event.read() {
            info!("SEND: {:?}", cmd);
            ws_session.send.push(cmd.write_to_vec().unwrap().into());
        }
    }
}

/// System for handling WebSocket events.
#[allow(clippy::too_many_arguments)]
fn handle_websocket_response(
    mut sessions: Query<(Entity, &Name, Option<&mut Session>)>,
    mut nm_state: ResMut<NextState<NetworkManagerState>>,
    mut create_room_event: EventWriter<CreateRoomEvent>,
    mut list_room_event: EventWriter<ListRoomEvent>,
    mut join_room_event: EventWriter<JoinRoomEvent>,
    mut game_state_event: EventWriter<GameStateEvent>,
    mut roll_event: EventWriter<RollEvent>,
    mut keep_dice_event: EventWriter<KeepDiceEvent>,
    mut change_turn_event: EventWriter<ChangeTurnEvent>,
    mut change_score_event: EventWriter<ChangeScoreEvent>,
    mut error_event: EventWriter<ErrorEvent>,
    mut room_info_event: EventWriter<RoomInfoEvent>,
) {
    if let Ok((_, _, Some(mut ws_session))) = sessions.single_mut() {
        for packet in ws_session.recv.drain(..) {
            if let Ok(res) = ServerAPIResponse::read_from_buffer(&packet.payload) {
                info!("RCV: {:?}", res);

                match res {
                    ServerAPIResponse::Ok => todo!(),
                    ServerAPIResponse::Ping(_) => todo!(),
                    ServerAPIResponse::Register(_) => {
                        nm_state.set(NetworkManagerState::Connected);
                    }
                    ServerAPIResponse::CreateRoom(create_room_response) => {
                        create_room_event.write(CreateRoomEvent(create_room_response));
                    }
                    ServerAPIResponse::ListRoom(list_room_response) => {
                        list_room_event.write(ListRoomEvent(list_room_response));
                    }
                    ServerAPIResponse::JoinRoom(join_room_response) => {
                        join_room_event.write(JoinRoomEvent(join_room_response));
                    }
                    ServerAPIResponse::GameState(game_state_response) => {
                        game_state_event.write(GameStateEvent(game_state_response));
                    }
                    ServerAPIResponse::Roll(roll_response) => {
                        roll_event.write(RollEvent(roll_response));
                    }
                    ServerAPIResponse::KeepDice(keep_dice_response) => {
                        keep_dice_event.write(KeepDiceEvent(keep_dice_response));
                    }
                    ServerAPIResponse::ChangeTurn(change_turn_response) => {
                        change_turn_event.write(ChangeTurnEvent(change_turn_response));
                    }
                    ServerAPIResponse::ChangeScore(change_score_response) => {
                        change_score_event.write(ChangeScoreEvent(change_score_response));
                    }
                    ServerAPIResponse::Error(error_response) => {
                        error_event.write(ErrorEvent(error_response));
                    }
                    ServerAPIResponse::RoomInfo(room_info_response) => {
                        room_info_event.write(RoomInfoEvent(room_info_response));
                    }
                };
            } else {
                error!("Response cannot be parsed!");
            }
        }
    }
}
