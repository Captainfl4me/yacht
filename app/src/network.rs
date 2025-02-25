
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
    state: NetworkManagerState,
    backend_url: &'a str,
}

impl<'a> NetworkManager<'a> {
    pub async fn new(backend_url: &'a str) -> Self {
        NetworkManager {
            state: NetworkManagerState::NotInit,
            backend_url,
        }
    }

    pub async fn udpate(&mut self) {
        match self.state {
            NetworkManagerState::NotInit => {
                self.state = NetworkManagerState::Connecting;
            }
            NetworkManagerState::Connecting => {
                //if self.ws.get_state() == WebSocketState::Opened {
                    self.state = NetworkManagerState::Connected;
                //}
            }
            NetworkManagerState::Connected => {
                // let val = futures_util::future::poll_fn(|ctx| {
                //     match self.ws.poll_next_unpin(ctx) {
                //         std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
                //         std::task::Poll::Ready(Some(msg)) => std::task::Poll::Ready(Some(msg)),
                //         std::task::Poll::Pending => std::task::Poll::Ready(None)
                //     }
                // }).await;
                //
                // if val.is_some() {
                //     println!("Recv");
                // }
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        //self.ws.get_state() == WebSocketState::Opened
        true
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

        //self.ws.send_binary(data.as_mut_slice());
    }
}
