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
pub struct StartGameResponse;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct RollResponse;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct KeepDiceResponse;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub enum ErrorResponse {
    NotRegister,
    NotFound,
    PlayerAlreadyInRoom,
    NotInRoom,
    NotEnoughPermission,
}

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub enum ServerAPIResponse {
    Ok,
    Ping(PingResponse),
    Register(RegisterResponse),
    CreateRoom(CreateRoomResponse),
    ListRoom(ListRoomResponse),
    JoinRoom(JoinRoomResponse),
    StartGame(StartGameResponse),
    Roll(RollResponse),
    KeepDice(KeepDiceResponse),
    Error(ErrorResponse)
}
