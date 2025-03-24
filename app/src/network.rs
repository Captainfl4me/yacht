use super::PlayerData;
use aeronet_io::{
    connection::{DisconnectReason, Disconnected},
    Session,
};
use aeronet_websocket::client::{ClientConfig, WebSocketClient, WebSocketClientPlugin};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use shared::{ServerAPICommand, ServerAPIResponse};
use speedy::{Readable, Writable};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum NetworkManagerState {
    #[default]
    Disconnect,
    Connecting,
    Connected,
}

#[derive(Resource)]
pub struct NetworkManager {
    pub send_queue: VecDeque<ServerAPICommand>,
    pub read_queue: VecDeque<ServerAPIResponse>,
}

impl NetworkManager {
    pub fn new() -> Self {
        NetworkManager {
            send_queue: VecDeque::new(),
            read_queue: VecDeque::new(),
        }
    }
}

pub fn network_plugin(app: &mut App) {
    app.init_state::<NetworkManagerState>()
        .add_plugins(WebSocketClientPlugin)
        .add_systems(Startup, ws_setup)
        .add_systems(
            Update,
            handle_websocket_events.run_if(not(in_state(NetworkManagerState::Disconnect))),
        )
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

fn ws_setup(mut commands: Commands, mut nm_state: ResMut<NextState<NetworkManagerState>>) {
    let target = "ws://127.0.0.1:8080";
    let config = client_config();

    let name = format!("{}. {target}", 0);
    commands
        .spawn(Name::new(name))
        .queue(WebSocketClient::connect(config, target));

    commands.insert_resource(NetworkManager::new());
    nm_state.set(NetworkManagerState::Connecting);
}

fn on_connected(
    trigger: Trigger<OnAdd, Session>,
    names: Query<&Name>,
    mut nm_state: ResMut<NextState<NetworkManagerState>>,
    mut nm: ResMut<NetworkManager>,
    player_data: Res<Persistent<PlayerData>>,
) {
    let entity = trigger.entity();
    let name = names
        .get(entity)
        .expect("our session entity should have a name");
    info!("{name} connected");

    nm_state.set(NetworkManagerState::Connected);
    nm.send_queue
        .push_back(ServerAPICommand::Register(shared::RegisterCommand(
            player_data.uuid,
        )));
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
fn handle_websocket_events(
    mut sessions: Query<(Entity, &Name, Option<&mut Session>)>,
    mut nm: ResMut<NetworkManager>,
    mut nm_state: ResMut<NextState<NetworkManagerState>>,
) {
    if let (_, _, Some(mut ws_session)) = sessions.single_mut() {
        for packet in ws_session.recv.drain(..) {
            if let Ok(res) = ServerAPIResponse::read_from_buffer(&packet.payload) {
                info!("RCV: {:?}", res);
                if let ServerAPIResponse::Register(shared::RegisterResponse) = res {
                    nm_state.set(NetworkManagerState::Connected);
                } else {
                    nm.read_queue.push_back(res);
                }
            } else {
                error!("Response cannot be parsed!");
            }
        }

        while let Some(cmd_to_send) = nm.send_queue.pop_front() {
            info!("SEND: {:?}", cmd_to_send);
            ws_session
                .send
                .push(cmd_to_send.write_to_vec().unwrap().into());
        }
    }
}
