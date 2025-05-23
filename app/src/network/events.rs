use bevy::prelude::*;
use shared::{
    ChangeScoreResponse, ChangeTurnResponse, CreateRoomResponse, ErrorResponse, JoinRoomResponse,
    KeepDiceResponse, ListRoomResponse, RollResponse, RoomInfoResponse, RoomUpdateResponse,
};

#[derive(Event, Debug)]
pub struct CreateRoomEvent(pub CreateRoomResponse);

#[derive(Event, Debug)]
pub struct ListRoomEvent(pub ListRoomResponse);

#[derive(Event, Debug)]
pub struct JoinRoomEvent(pub JoinRoomResponse);

#[derive(Event, Debug)]
pub struct RoomUpdateEvent(pub RoomUpdateResponse);

#[derive(Event, Debug)]
pub struct RollEvent(pub RollResponse);

#[derive(Event, Debug)]
pub struct KeepDiceEvent(pub KeepDiceResponse);

#[derive(Event, Debug)]
pub struct ChangeTurnEvent(pub ChangeTurnResponse);

#[derive(Event, Debug)]
pub struct ChangeScoreEvent(pub ChangeScoreResponse);

#[derive(Event, Debug)]
pub struct ErrorEvent(pub ErrorResponse);

#[derive(Event, Debug)]
pub struct RoomInfoEvent(pub RoomInfoResponse);
