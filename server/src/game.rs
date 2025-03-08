use log::*;
use shared::{ServerAPICommand, ServerAPIResponse};
use speedy::Readable;

pub fn handle_request(msg: &[u8]) -> Option<ServerAPIResponse> {
    info!("Raw: {:?}", msg);

    if let Ok(cmd) = ServerAPICommand::read_from_buffer(msg) {
        let res = match cmd {
            ServerAPICommand::Ping(ping_cmd) => {
                info!("Receive Ping");
                ServerAPIResponse::Ping(shared::PingResponse)
            }
            ServerAPICommand::CreateRoom(create_room_cmd) => {
                info!("Receive CreateRoom");
                ServerAPIResponse::CreateRoom(shared::CreateRoomResponse)
            }
            ServerAPICommand::ListRoom(list_room_cmd) => {
                info!("Receive ListRoom");
                ServerAPIResponse::ListRoom(shared::ListRoomResponse(Vec::new()))
            }
            ServerAPICommand::JoinRoom(join_room_cmd) => {
                info!("Receive JoinRoom");
                ServerAPIResponse::JoinRoom(shared::JoinRoomResponse)
            }
            ServerAPICommand::Roll(roll_cmd) => {
                info!("Receive Roll");
                ServerAPIResponse::Roll(shared::RollResponse)
            }
            ServerAPICommand::KeepDice(keep_dice_cmd) => {
                info!("Receive KeepDice");
                ServerAPIResponse::KeepDice(shared::KeepDiceResponse)
            }
        };

        Some(res)
    } else {
        error!("Response cannot be parsed!");
        None
    }
}
