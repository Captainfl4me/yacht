#[cfg(target_os = "emscripten")]
use emscripten_functions::websocket::*;

pub enum ServerAPICommand {
    Ping,
    CreateRoom,
    ListRoom,
    JoinRoom,
    Roll,
    KeepDice,
}

pub enum NetworkManagerState {
    NotInit,
    Connecting,
    Connected,
}

pub struct NetworkManager<'a> {
    ws: WebSocket,
    state: NetworkManagerState,
    backend_url: &'a str,
}

impl<'a> NetworkManager<'a> {
    pub fn new(backend_url: &'a str) -> Self{
        NetworkManager {
            ws: WebSocket::new().unwrap(),
            state: NetworkManagerState::NotInit,
            backend_url
        }
    }
    
    pub fn udpate(&mut self) {
        match self.state {
            NetworkManagerState::NotInit => {
                self.ws.connect(self.backend_url);
                self.state = NetworkManagerState::Connecting;
            },
            NetworkManagerState::Connecting => {
                if self.ws.get_state() == WebSocketState::Opened {
                    self.state = NetworkManagerState::Connected;
                }
            },
            NetworkManagerState::Connected => {},
        }
    }

    pub fn is_connected(&self) -> bool {
        self.ws.get_state() == WebSocketState::Opened
    }

    pub fn send_command(&mut self, command: ServerAPICommand) {
        let mut data: Vec<u8> = match command {
            ServerAPICommand::Ping => vec![0u8],
            ServerAPICommand::CreateRoom => vec![1u8],
            ServerAPICommand::ListRoom => vec![2u8],
            ServerAPICommand::JoinRoom => vec![3u8],
            ServerAPICommand::Roll => vec![4u8],
            ServerAPICommand::KeepDice => vec![5u8],
        };

        self.ws.send_binary(data.as_mut_slice());
    }
}
