use speedy::{Readable, Writable};
use uuid::Uuid;
use super::RoomHeader;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct PingResponse;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct RegisterResponse;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct CreateRoomResponse(pub Uuid);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct ListRoomResponse(pub Vec<RoomHeader>);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct JoinRoomResponse(pub Uuid);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct RollResponse;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct KeepDiceResponse;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub enum ErrorResponse {
    NotRegister,
    NotFound,
    PlayerAlreadyInRoom
}

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
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
