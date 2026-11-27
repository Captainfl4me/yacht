use futures_util::{SinkExt, StreamExt};
use shared::{ServerAPICommand, ServerAPIResponse};
use speedy::{Readable, Writable};
use std::collections::VecDeque;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tokio_tungstenite::MaybeTlsStream;

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// The server under test binds a fixed port, so e2e tests must run one at a time.
static E2E_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Dedicated address for the server under test, so it never clashes with a
/// server that may already be running on the default port.
const TEST_SERVER_ADDR: &str = "127.0.0.1:18080";

fn e2e_lock() -> std::sync::MutexGuard<'static, ()> {
    E2E_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

struct ServerGuard(Child);

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct Client {
    ws: Ws,
    inbox: VecDeque<ServerAPIResponse>,
}

impl Client {
    async fn connect() -> Self {
        let (ws, _) = connect_async(&format!("ws://{}", TEST_SERVER_ADDR))
            .await
            .expect("connect");
        Client {
            ws,
            inbox: VecDeque::new(),
        }
    }

    async fn send(&mut self, cmd: ServerAPICommand) {
        self.ws
            .send(Message::Binary(cmd.write_to_vec().unwrap().into()))
            .await
            .unwrap();
    }

    async fn next(&mut self) -> ServerAPIResponse {
        loop {
            match self.ws.next().await {
                Some(Ok(Message::Binary(bytes))) => {
                    return ServerAPIResponse::read_from_buffer(&bytes).expect("parse response");
                }
                Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => continue,
                other => panic!("unexpected frame: {:?}", other.map(|m| m.map(|_| ()))),
            }
        }
    }

    async fn wait_for(
        &mut self,
        predicate: impl Fn(&ServerAPIResponse) -> bool,
        what: &str,
    ) -> ServerAPIResponse {
        if let Some(pos) = self.inbox.iter().position(|m| predicate(m)) {
            return self.inbox.remove(pos).unwrap();
        }
        for _ in 0..50 {
            let msg = self.next().await;
            if predicate(&msg) {
                return msg;
            }
            self.inbox.push_back(msg);
        }
        panic!("expected {} but it did not arrive", what);
    }
}

async fn start_server() -> ServerGuard {
    let child = Command::new(env!("CARGO_BIN_EXE_yatch-server"))
        .env("YATCH_SERVER_ADDR", TEST_SERVER_ADDR)
        // Short grace period so disconnect-expiry tests stay fast.
        .env("YATCH_DISCONNECT_GRACE_SECS", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn yatch-server");
    let guard = ServerGuard(child);
    for _ in 0..50 {
        if TcpStream::connect(TEST_SERVER_ADDR).await.is_ok() {
            return guard;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("server did not start listening on {}", TEST_SERVER_ADDR);
}

fn register(uuid: uuid::Uuid, name: &str) -> ServerAPICommand {
    ServerAPICommand::Register(shared::RegisterCommand(
        shared::PROTOCOL_VERSION,
        uuid,
        name.to_string(),
    ))
}

#[tokio::test]
async fn e2e_two_player_game_flow() {
    let _lock = e2e_lock();
    let _server = start_server().await;

    let mut alice = Client::connect().await;
    let mut bob = Client::connect().await;

    let alice_uuid = uuid::Uuid::new_v4();
    let bob_uuid = uuid::Uuid::new_v4();

    alice.send(register(alice_uuid, "Alice")).await;
    alice
        .wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "Register")
        .await;
    bob.send(register(bob_uuid, "Bob")).await;
    bob.wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "Register")
        .await;

    bob.send(ServerAPICommand::Register(shared::RegisterCommand(
        255,
        bob_uuid,
        "X".to_string(),
    )))
    .await;
    bob.wait_for(
        |r| matches!(r, ServerAPIResponse::Error(shared::ErrorResponse::VersionMismatch)),
        "VersionMismatch",
    )
    .await;

    alice
        .send(ServerAPICommand::CreateRoom(shared::CreateRoomCommand(
            "e2e".to_string(),
        )))
        .await;
    let room_uuid = match alice
        .wait_for(|r| matches!(r, ServerAPIResponse::CreateRoom(_)), "CreateRoom")
        .await
    {
        ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(uuid)) => uuid,
        other => panic!("expected CreateRoom, got {:?}", other),
    };

    bob.send(ServerAPICommand::ListRoom(shared::ListRoomCommand)).await;
    match bob
        .wait_for(|r| matches!(r, ServerAPIResponse::ListRoom(_)), "ListRoom")
        .await
    {
        ServerAPIResponse::ListRoom(shared::ListRoomResponse(rooms)) => {
            assert!(rooms.iter().any(|r| r.uuid == room_uuid && r.name == "e2e"));
        }
        other => panic!("expected ListRoom, got {:?}", other),
    }

    bob.send(ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)))
        .await;
    bob.wait_for(|r| matches!(r, ServerAPIResponse::JoinRoom(_)), "JoinRoom")
        .await;

    match alice
        .wait_for(
            |r| matches!(r, ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(shared::RoomUpdateReason::NewPlayer(_)))),
            "NewPlayer broadcast",
        )
        .await
    {
        ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
            shared::RoomUpdateReason::NewPlayer(player),
        )) => {
            assert_eq!(player.uuid, bob_uuid);
            assert_eq!(player.name, "Bob");
        }
        other => panic!("expected NewPlayer, got {:?}", other),
    }

    bob.send(ServerAPICommand::StartGame(shared::StartGameCommand)).await;
    bob.wait_for(
        |r| matches!(r, ServerAPIResponse::Error(shared::ErrorResponse::NotEnoughPermission)),
        "NotEnoughPermission",
    )
    .await;

    alice
        .send(ServerAPICommand::StartGame(shared::StartGameCommand))
        .await;
    alice.wait_for(|r| r == &ServerAPIResponse::Ok, "Ok").await;
    for client in [&mut alice, &mut bob] {
        client
            .wait_for(
                |r| {
                    matches!(
                        r,
                        ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
                            shared::RoomUpdateReason::GameStateChange(shared::GameState::Started)
                        ))
                    )
                },
                "GameStateChange(Started)",
            )
            .await;
    }

    bob.send(ServerAPICommand::Roll(shared::RollCommand)).await;
    bob.wait_for(
        |r| matches!(r, ServerAPIResponse::Error(shared::ErrorResponse::NotEnoughPermission)),
        "NotEnoughPermission for roll",
    )
    .await;

    alice.send(ServerAPICommand::Roll(shared::RollCommand)).await;
    alice.wait_for(|r| r == &ServerAPIResponse::Ok, "Ok").await;
    for client in [&mut alice, &mut bob] {
        match client
            .wait_for(|r| matches!(r, ServerAPIResponse::Roll(_)), "Roll broadcast")
            .await
        {
            ServerAPIResponse::Roll(shared::RollResponse(dices)) => {
                assert!(dices.iter().all(|d| (1..=6).contains(d)));
            }
            other => panic!("expected Roll, got {:?}", other),
        }
    }

    alice
        .send(ServerAPICommand::KeepDice(shared::KeepDiceCommand([1, 1, 0, 0, 0])))
        .await;
    alice.wait_for(|r| r == &ServerAPIResponse::Ok, "Ok").await;
    for client in [&mut alice, &mut bob] {
        match client
            .wait_for(|r| matches!(r, ServerAPIResponse::KeepDice(_)), "KeepDice broadcast")
            .await
        {
            ServerAPIResponse::KeepDice(shared::KeepDiceResponse(mask)) => {
                assert_eq!(mask, [1, 1, 0, 0, 0])
            }
            other => panic!("expected KeepDice, got {:?}", other),
        }
    }

    alice
        .send(ServerAPICommand::SelectPoints(shared::SelectPointsCommand(
            shared::Score::Aces(0),
        )))
        .await;
    alice.wait_for(|r| r == &ServerAPIResponse::Ok, "Ok").await;
    for client in [&mut alice, &mut bob] {
        match client
            .wait_for(|r| matches!(r, ServerAPIResponse::ChangeScore(_)), "ChangeScore")
            .await
        {
            ServerAPIResponse::ChangeScore(shared::ChangeScoreResponse(
                player,
                shared::Score::Aces(_),
            )) => {
                assert_eq!(player, 0);
            }
            other => panic!("expected ChangeScore, got {:?}", other),
        }
        match client
            .wait_for(|r| matches!(r, ServerAPIResponse::ChangeTurn(_)), "ChangeTurn")
            .await
        {
            ServerAPIResponse::ChangeTurn(shared::ChangeTurnResponse(turn)) => assert_eq!(turn, 1),
            other => panic!("expected ChangeTurn, got {:?}", other),
        }
    }

    let mut carol = Client::connect().await;
    let carol_uuid = uuid::Uuid::new_v4();
    carol.send(register(carol_uuid, "Carol")).await;
    carol
        .wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "Register")
        .await;
    carol
        .send(ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)))
        .await;
    carol
        .wait_for(
            |r| matches!(r, ServerAPIResponse::Error(shared::ErrorResponse::GameAlreadyStarted)),
            "GameAlreadyStarted",
        )
        .await;

    alice
        .send(ServerAPICommand::LeaveRoom(shared::LeaveRoomCommand))
        .await;
    alice
        .wait_for(|r| matches!(r, ServerAPIResponse::LeaveRoom(_)), "LeaveRoom")
        .await;
    match bob
        .wait_for(
            |r| {
                matches!(
                    r,
                    ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
                        shared::RoomUpdateReason::PlayerLeft(_)
                    ))
                )
            },
            "PlayerLeft",
        )
        .await
    {
        ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
            shared::RoomUpdateReason::PlayerLeft(uuid),
        )) => assert_eq!(uuid, alice_uuid),
        other => panic!("expected PlayerLeft, got {:?}", other),
    }
}

#[tokio::test]
async fn e2e_guards_restart_and_disconnect() {
    let _lock = e2e_lock();
    let _server = start_server().await;

    let mut alice = Client::connect().await;
    let mut bob = Client::connect().await;
    let alice_uuid = uuid::Uuid::new_v4();
    let bob_uuid = uuid::Uuid::new_v4();

    alice.send(register(alice_uuid, "Alice")).await;
    alice
        .wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "Register")
        .await;
    bob.send(register(bob_uuid, "Bob")).await;
    bob.wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "Register")
        .await;

    // Joining a room that does not exist -> NotFound
    bob.send(ServerAPICommand::JoinRoom(shared::JoinRoomCommand(
        uuid::Uuid::new_v4(),
    )))
    .await;
    bob.wait_for(
        |r| matches!(r, ServerAPIResponse::Error(shared::ErrorResponse::NotFound)),
        "NotFound for unknown room",
    )
    .await;

    alice
        .send(ServerAPICommand::CreateRoom(shared::CreateRoomCommand(
            "e2e".to_string(),
        )))
        .await;
    let room_uuid = match alice
        .wait_for(|r| matches!(r, ServerAPIResponse::CreateRoom(_)), "CreateRoom")
        .await
    {
        ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(uuid)) => uuid,
        other => panic!("expected CreateRoom, got {:?}", other),
    };

    bob.send(ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)))
        .await;
    bob.wait_for(|r| matches!(r, ServerAPIResponse::JoinRoom(_)), "JoinRoom")
        .await;
    alice
        .wait_for(
            |r| {
                matches!(
                    r,
                    ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
                        shared::RoomUpdateReason::NewPlayer(_)
                    ))
                )
            },
            "NewPlayer broadcast",
        )
        .await;

    // Rolling before the game has started is rejected, even for the creator
    alice
        .send(ServerAPICommand::Roll(shared::RollCommand))
        .await;
    alice
        .wait_for(
            |r| matches!(r, ServerAPIResponse::Error(shared::ErrorResponse::NotEnoughPermission)),
            "NotEnoughPermission for roll before start",
        )
        .await;

    // Start the game, then the creator restarts it -> NotStarted for everyone
    alice
        .send(ServerAPICommand::StartGame(shared::StartGameCommand))
        .await;
    alice.wait_for(|r| r == &ServerAPIResponse::Ok, "Ok").await;
    for client in [&mut alice, &mut bob] {
        client
            .wait_for(
                |r| {
                    matches!(
                        r,
                        ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
                            shared::RoomUpdateReason::GameStateChange(shared::GameState::Started)
                        ))
                    )
                },
                "GameStateChange(Started)",
            )
            .await;
    }

    alice
        .send(ServerAPICommand::RestartGame(shared::RestartGameCommand))
        .await;
    alice
        .wait_for(|r| r == &ServerAPIResponse::Ok, "Ok for restart")
        .await;
    for client in [&mut alice, &mut bob] {
        client
            .wait_for(
                |r| {
                    matches!(
                        r,
                        ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
                            shared::RoomUpdateReason::GameStateChange(shared::GameState::NotStarted)
                        ))
                    )
                },
                "GameStateChange(NotStarted)",
            )
            .await;
    }

    // When a client disconnects, the server keeps its seat during the grace
    // period, then removes it from the room and notifies the remaining players
    drop(bob);
    match alice
        .wait_for(
            |r| {
                matches!(
                    r,
                    ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
                        shared::RoomUpdateReason::PlayerLeft(_)
                    ))
                )
            },
            "PlayerLeft after disconnect",
        )
        .await
    {
        ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
            shared::RoomUpdateReason::PlayerLeft(uuid),
        )) => assert_eq!(uuid, bob_uuid),
        other => panic!("expected PlayerLeft, got {:?}", other),
    }
}

/// Page reload during a started game: the old socket drops and the "reloaded
/// page" re-registers the same uuid within the grace period. The player must
/// keep its seat, get the room id back in the register response, and receive
/// room broadcasts again on the new socket.
#[tokio::test]
async fn e2e_reload_rejoin_within_grace() {
    let _lock = e2e_lock();
    let _server = start_server().await;

    let alice_uuid = uuid::Uuid::new_v4();
    let bob_uuid = uuid::Uuid::new_v4();

    let mut alice = Client::connect().await;
    alice.send(register(alice_uuid, "Alice")).await;
    alice
        .wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "Register")
        .await;
    alice
        .send(ServerAPICommand::CreateRoom(shared::CreateRoomCommand(
            "e2e".to_string(),
        )))
        .await;
    let room_uuid = match alice
        .wait_for(|r| matches!(r, ServerAPIResponse::CreateRoom(_)), "CreateRoom")
        .await
    {
        ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(uuid)) => uuid,
        other => panic!("expected CreateRoom, got {:?}", other),
    };

    let mut bob = Client::connect().await;
    bob.send(register(bob_uuid, "Bob")).await;
    bob.wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "Register")
        .await;
    bob.send(ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)))
        .await;
    bob.wait_for(|r| matches!(r, ServerAPIResponse::JoinRoom(_)), "JoinRoom")
        .await;

    alice
        .send(ServerAPICommand::StartGame(shared::StartGameCommand))
        .await;
    alice.wait_for(|r| r == &ServerAPIResponse::Ok, "Ok").await;
    for client in [&mut alice, &mut bob] {
        client
            .wait_for(
                |r| {
                    matches!(
                        r,
                        ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
                            shared::RoomUpdateReason::GameStateChange(shared::GameState::Started)
                        ))
                    )
                },
                "GameStateChange(Started)",
            )
            .await;
    }

    // === page reload: old socket drops, new page re-registers same uuid ===
    drop(alice);
    let mut alice2 = Client::connect().await;
    alice2.send(register(alice_uuid, "Alice")).await;
    match alice2
        .wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "re-Register")
        .await
    {
        ServerAPIResponse::Register(shared::RegisterResponse(room)) => {
            assert_eq!(room, Some(room_uuid), "re-register should report the room");
        }
        other => panic!("expected Register, got {:?}", other),
    }

    // The rejoined socket can query the room and still sees both players
    alice2
        .send(ServerAPICommand::RoomInfo(shared::RoomInfoCommand))
        .await;
    match alice2
        .wait_for(|r| matches!(r, ServerAPIResponse::RoomInfo(_)), "RoomInfo")
        .await
    {
        ServerAPIResponse::RoomInfo(shared::RoomInfoResponse(info)) => {
            let uuids: Vec<_> = info.players.iter().map(|p| p.uuid).collect();
            assert!(uuids.contains(&alice_uuid), "alice still in room: {:?}", uuids);
            assert!(uuids.contains(&bob_uuid));
            assert_eq!(info.game_state, shared::GameState::Started);
        }
        other => panic!("expected RoomInfo, got {:?}", other),
    }

    // The rejoined socket receives room broadcasts again (bob leaves)
    bob.send(ServerAPICommand::LeaveRoom(shared::LeaveRoomCommand))
        .await;
    bob.wait_for(
        |r| matches!(r, ServerAPIResponse::LeaveRoom(_)),
        "LeaveRoom",
    )
    .await;
    match alice2
        .wait_for(
            |r| {
                matches!(
                    r,
                    ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
                        shared::RoomUpdateReason::PlayerLeft(_)
                    ))
                )
            },
            "PlayerLeft broadcast on re-attached socket",
        )
        .await
    {
        ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
            shared::RoomUpdateReason::PlayerLeft(uuid),
        )) => assert_eq!(uuid, bob_uuid),
        other => panic!("expected PlayerLeft, got {:?}", other),
    }

    // A brand-new player still cannot join a started game
    let mut carol = Client::connect().await;
    carol.send(register(uuid::Uuid::new_v4(), "Carol")).await;
    carol
        .wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "Register")
        .await;
    carol
        .send(ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)))
        .await;
    carol
        .wait_for(
            |r| matches!(r, ServerAPIResponse::Error(shared::ErrorResponse::GameAlreadyStarted)),
            "GameAlreadyStarted for new player",
        )
        .await;
}

/// Two tabs of the same browser share the persisted uuid: the second tab
/// registers the same uuid on a second socket. Closing the *first* tab must
/// not eject the player while the second tab is alive (stale socket); closing
/// the *last* socket must eject the player after the grace period.
#[tokio::test]
async fn e2e_second_tab_stale_socket_then_grace_expiry() {
    let _lock = e2e_lock();
    let _server = start_server().await;

    let alice_uuid = uuid::Uuid::new_v4();
    let bob_uuid = uuid::Uuid::new_v4();

    // Tab A
    let mut tab_a = Client::connect().await;
    tab_a.send(register(alice_uuid, "Alice")).await;
    tab_a
        .wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "Register")
        .await;
    tab_a
        .send(ServerAPICommand::CreateRoom(shared::CreateRoomCommand(
            "e2e".to_string(),
        )))
        .await;
    let room_uuid = match tab_a
        .wait_for(|r| matches!(r, ServerAPIResponse::CreateRoom(_)), "CreateRoom")
        .await
    {
        ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(uuid)) => uuid,
        other => panic!("expected CreateRoom, got {:?}", other),
    };

    let mut bob = Client::connect().await;
    bob.send(register(bob_uuid, "Bob")).await;
    bob.wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "Register")
        .await;
    bob.send(ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)))
        .await;
    bob.wait_for(|r| matches!(r, ServerAPIResponse::JoinRoom(_)), "JoinRoom")
        .await;

    // Tab B: same browser => same persisted uuid, second simultaneous socket
    let mut tab_b = Client::connect().await;
    tab_b.send(register(alice_uuid, "Alice")).await;
    match tab_b
        .wait_for(|r| matches!(r, ServerAPIResponse::Register(_)), "Register tab B")
        .await
    {
        ServerAPIResponse::Register(shared::RegisterResponse(room)) => {
            assert_eq!(room, Some(room_uuid));
        }
        other => panic!("expected Register, got {:?}", other),
    }

    // Close tab A (the stale socket). Even after the grace period has elapsed,
    // the player must still be in the room because tab B owns the current epoch.
    drop(tab_a);
    tokio::time::sleep(Duration::from_millis(2500)).await;
    bob.send(ServerAPICommand::RoomInfo(shared::RoomInfoCommand))
        .await;
    match bob
        .wait_for(|r| matches!(r, ServerAPIResponse::RoomInfo(_)), "RoomInfo")
        .await
    {
        ServerAPIResponse::RoomInfo(shared::RoomInfoResponse(info)) => {
            let uuids: Vec<_> = info.players.iter().map(|p| p.uuid).collect();
            assert!(
                uuids.contains(&alice_uuid),
                "stale tab close must not eject the player: {:?}",
                uuids
            );
        }
        other => panic!("expected RoomInfo, got {:?}", other),
    }

    // Close tab B (the current socket): after the grace period the player is
    // removed and the remaining player is notified.
    drop(tab_b);
    match bob
        .wait_for(
            |r| {
                matches!(
                    r,
                    ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
                        shared::RoomUpdateReason::PlayerLeft(_)
                    ))
                )
            },
            "PlayerLeft after grace expiry",
        )
        .await
    {
        ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(
            shared::RoomUpdateReason::PlayerLeft(uuid),
        )) => assert_eq!(uuid, alice_uuid),
        other => panic!("expected PlayerLeft, got {:?}", other),
    }
}
