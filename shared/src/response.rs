use speedy::{Readable, Writable};
use super::RoomHeader;

#[derive(Debug, Readable, Writable)]
pub struct PingResponse;

#[derive(Debug, Readable, Writable)]
pub struct RegisterResponse;

#[derive(Debug, Readable, Writable)]
pub struct CreateRoomResponse;

#[derive(Debug, Readable, Writable)]
pub struct ListRoomResponse(pub Vec<RoomHeader>);

#[derive(Debug, Readable, Writable)]
pub struct JoinRoomResponse;

#[derive(Debug, Readable, Writable)]
pub struct RollResponse;

#[derive(Debug, Readable, Writable)]
pub struct KeepDiceResponse;

#[derive(Debug, Readable, Writable)]
pub enum ErrorResponse {
    NotRegister
}

#[derive(Debug, Readable, Writable)]
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
