use aeronet_io::{
    connection::{DisconnectReason, Disconnected},
    Session,
};
use aeronet_websocket::client::{ClientConfig, WebSocketClient, WebSocketClientPlugin};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Serialize, Deserialize)]
pub enum ServerAPICommand {
    Ping,
    CreateRoom,
    ListRoom,
    JoinRoom,
    Roll,
    KeepDice,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ServerAPIResponse {
    Ping,
    CreateRoom,
    ListRoom,
    JoinRoom,
    Roll,
    KeepDice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum NetworkManagerState {
    Disconnect,
    #[default]
    Connecting,
    Connected,
}

#[derive(Resource)]
pub struct NetworkManager {
    send_queue: VecDeque<ServerAPICommand>,
    read_queue: VecDeque<ServerAPIResponse>,
}

pub fn network_plugin(app: &mut App) {
    app.init_state::<NetworkManagerState>()
        .add_plugins(WebSocketClientPlugin)
        .add_systems(Startup, ws_setup)
        .add_systems(Update, handle_websocket_events)
        .add_observer(on_connected)
        .add_observer(on_disconnected);
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

fn ws_setup(mut commands: Commands) {
    let target = "ws://127.0.0.1:8080";
    let config = client_config();

    let name = format!("{}. {target}", 0);
    commands
        .spawn(Name::new(name))
        .queue(WebSocketClient::connect(config, target));
}

fn on_connected(
    trigger: Trigger<OnAdd, Session>,
    names: Query<&Name>,
    mut nm_state: ResMut<NextState<NetworkManagerState>>,
) {
    let entity = trigger.entity();
    let name = names
        .get(entity)
        .expect("our session entity should have a name");
    info!("{name} connected");
    nm_state.set(NetworkManagerState::Connected);
}

fn on_disconnected(
    trigger: Trigger<Disconnected>,
    names: Query<&Name>,
    mut nm_state: ResMut<NextState<NetworkManagerState>>,
) {
    let entity = trigger.entity();
    let name = names
        .get(entity)
        .expect("our session entity should have a name");
    info!(
        "{name} disconnected: {}",
        match &trigger.reason {
            DisconnectReason::User(reason) => {
                format!("by user: {reason}")
            }
            DisconnectReason::Peer(reason) => {
                format!("by peer: {reason}")
            }
            DisconnectReason::Error(err) => {
                format!("due to error: {err:?}")
            }
        }
    );

    nm_state.set(NetworkManagerState::Disconnect);
}

/// System for handling WebSocket events.
fn handle_websocket_events(mut sessions: Query<&mut Session>, mut nm: ResMut<NetworkManager>) {
    let mut ws_session = sessions.single_mut();

    for packet in ws_session.recv.drain(..) {
        if let Ok(res) = serde_bencode::from_bytes::<ServerAPIResponse>(&packet.payload) {
            nm.read_queue.push_back(res);
        } else {
            error!("Response cannot be parsed!");
        }
    }

    while let Some(cmd_to_send) = nm.send_queue.pop_front() {
        ws_session
            .send
            .push(serde_bencode::to_bytes(&cmd_to_send).unwrap().into());
    }
}
