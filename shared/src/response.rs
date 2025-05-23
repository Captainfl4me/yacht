use speedy::{Readable, Writable};
use uuid::Uuid;
use crate::{PlayerHeader, RoomInfo};

use super::{RoomHeader, GameState};

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
pub struct RollResponse(pub [u8; 5]);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct KeepDiceResponse(pub [u8; 5]);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct ChangeTurnResponse(pub u8);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct ChangeScoreResponse(pub u8, pub crate::Score);

#[derive(Debug, Readable, Writable, PartialEq, Eq, Clone)]
pub enum RoomUpdateReason {
    GameStateChange(GameState),
    NewPlayer(PlayerHeader),
    PlayerLeft(Uuid)
}

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct RoomUpdateResponse(pub RoomUpdateReason);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub enum ErrorResponse {
    NotRegister,
    NotFound,
    PlayerAlreadyInRoom,
    NotInRoom,
    NotEnoughPermission,
}

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct RoomInfoResponse(pub RoomInfo);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub enum ServerAPIResponse {
    Ok,
    Ping(PingResponse),
    Register(RegisterResponse),
    CreateRoom(CreateRoomResponse),
    ListRoom(ListRoomResponse),
    JoinRoom(JoinRoomResponse),
    Roll(RollResponse),
    KeepDice(KeepDiceResponse),
    ChangeTurn(ChangeTurnResponse),
    ChangeScore(ChangeScoreResponse),
    RoomUpdate(RoomUpdateResponse),
    Error(ErrorResponse),
    RoomInfo(RoomInfoResponse)
}
