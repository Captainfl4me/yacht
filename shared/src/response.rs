use speedy::{Readable, Writable};
use super::RoomHeader;

#[derive(Readable, Writable)]
pub struct PingResponse;

#[derive(Readable, Writable)]
pub struct RegisterResponse;

#[derive(Readable, Writable)]
pub struct CreateRoomResponse;

#[derive(Readable, Writable)]
pub struct ListRoomResponse(pub Vec<RoomHeader>);

#[derive(Readable, Writable)]
pub struct JoinRoomResponse;

#[derive(Readable, Writable)]
pub struct RollResponse;

#[derive(Readable, Writable)]
pub struct KeepDiceResponse;

#[derive(Readable, Writable)]
pub enum ErrorResponse {
    NotRegister
}

#[derive(Readable, Writable)]
pub enum ServerAPIResponse {
    Ping(PingResponse),
    Register(RegisterResponse),
    CreateRoom(CreateRoomResponse),
    ListRoom(ListRoomResponse),
    JoinRoom(JoinRoomResponse),
    Roll(RollResponse),
    KeepDice(KeepDiceResponse),
    Error(ErrorResponse)
}
