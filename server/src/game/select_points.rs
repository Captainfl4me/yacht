use super::Handler;
use shared::{
    count_points_for_fullhouse, count_points_for_identical, count_points_for_numbers,
    count_points_for_straight, ErrorResponse, GameState, SelectPointsCommand, ServerAPIResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;

impl Handler for SelectPointsCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        data: &mut super::super::SocketLinkedData,
    ) -> shared::ServerAPIResponse {
        if let Some(player_uuid) = &data.uuid {
            let mut gs = game_state.lock().await;
            if gs.players.contains_key(player_uuid) {
                if let Some(room_id) = gs.players.get(player_uuid).unwrap().room {
                    if gs.rooms.contains_key(&room_id) {
                        let turn = gs.rooms.get(&room_id).unwrap().turn;

                        if gs.rooms.get(&room_id).unwrap().players[turn as usize] == *player_uuid
                            && gs.rooms.get(&room_id).unwrap().started
                        {
                            let mut score_set = false;
                            let mut game_ended = false;
                            gs.rooms.entry(room_id).and_modify(|room| {
                                if room.dices.iter().all(|dice| *dice == 0) {
                                    return;
                                }

                                let current_score = room.scores.get_mut(turn as usize).unwrap();
                                match self.0 {
                                    shared::Score::Aces(_) => {
                                        if current_score.aces.is_none() {
                                            let score = count_points_for_numbers(&room.dices, 1);
                                            current_score.aces = Some(score);
                                            if let Err(e) = room
                                                .change_score
                                                .send((turn, shared::Score::Aces(score)))
                                            {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Twos(_) => {
                                        if current_score.twos.is_none() {
                                            let score = count_points_for_numbers(&room.dices, 2);
                                            current_score.twos = Some(score);
                                            if let Err(e) = room
                                                .change_score
                                                .send((turn, shared::Score::Twos(score)))
                                            {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Threes(_) => {
                                        if current_score.threes.is_none() {
                                            let score = count_points_for_numbers(&room.dices, 3);
                                            current_score.threes = Some(score);
                                            if let Err(e) = room
                                                .change_score
                                                .send((turn, shared::Score::Threes(score)))
                                            {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Fours(_) => {
                                        if current_score.fours.is_none() {
                                            let score = count_points_for_numbers(&room.dices, 4);
                                            current_score.fours = Some(score);
                                            if let Err(e) = room
                                                .change_score
                                                .send((turn, shared::Score::Fours(score)))
                                            {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Fives(_) => {
                                        if current_score.fives.is_none() {
                                            let score = count_points_for_numbers(&room.dices, 5);
                                            current_score.fives = Some(score);
                                            if let Err(e) = room
                                                .change_score
                                                .send((turn, shared::Score::Fives(score)))
                                            {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Sixes(_) => {
                                        if current_score.sixes.is_none() {
                                            let score = count_points_for_numbers(&room.dices, 6);
                                            current_score.sixes = Some(score);
                                            if let Err(e) = room
                                                .change_score
                                                .send((turn, shared::Score::Sixes(score)))
                                            {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::ThreeOfAKind(_) => {
                                        if current_score.three_of_a_kind.is_none() {
                                            let score = count_points_for_identical(&room.dices, 3);
                                            current_score.three_of_a_kind = Some(score);
                                            if let Err(e) = room.change_score.send((
                                                turn,
                                                shared::Score::ThreeOfAKind(score),
                                            )) {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::FourOfAKind(_) => {
                                        if current_score.four_of_a_kind.is_none() {
                                            let score = count_points_for_identical(&room.dices, 4);
                                            current_score.four_of_a_kind = Some(score);
                                            if let Err(e) = room.change_score.send((
                                                turn,
                                                shared::Score::FourOfAKind(score),
                                            )) {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Fullhouse(_) => {
                                        if current_score.fullhouse.is_none() {
                                            let score = count_points_for_fullhouse(&room.dices);
                                            current_score.fullhouse = Some(score);
                                            if let Err(e) = room
                                                .change_score
                                                .send((turn, shared::Score::Fullhouse(score)))
                                            {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::SmallStraight(_) => {
                                        if current_score.small_straight.is_none() {
                                            let score = count_points_for_straight(&room.dices, 4);
                                            current_score.small_straight = Some(score);
                                            if let Err(e) = room.change_score.send((
                                                turn,
                                                shared::Score::SmallStraight(score),
                                            )) {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::LargeStraight(_) => {
                                        if current_score.large_straight.is_none() {
                                            let score = count_points_for_straight(&room.dices, 5);
                                            current_score.large_straight = Some(score);
                                            if let Err(e) = room.change_score.send((
                                                turn,
                                                shared::Score::LargeStraight(score),
                                            )) {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Yacht(_) => {
                                        let score = count_points_for_identical(&room.dices, 5);
                                        if current_score.yacht.is_none() {
                                            current_score.yacht = Some(score);
                                            if let Err(e) = room
                                                .change_score
                                                .send((turn, shared::Score::Yacht(score)))
                                            {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        } else if score == shared::YACHT_BONUS as u8 {
                                            current_score.extra_yachts += 1;
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Chance(_) => {
                                        if current_score.chance.is_none() {
                                            let score = count_points_for_identical(&room.dices, 1);
                                            current_score.chance = Some(score);
                                            if let Err(e) = room
                                                .change_score
                                                .send((turn, shared::Score::Chance(score)))
                                            {
                                                log::warn!("Failed to broadcast ChangeScore: {}", e);
                                            }
                                            score_set = true;
                                        }
                                    }
                                }

                                if score_set {
                                    room.throw_cnt = 0;
                                    room.turn = (room.turn + 1) % (room.players.len() as u8);
                                    if room.scores.iter().all(|score| score.is_full()) {
                                        game_ended = true;
                                    } else {
                                        room.dices = [0; 5];
                                        room.dices_mask = [0; 5];
                                        if let Err(e) = room.change_turn.send(room.turn as usize) {
                                            log::warn!("Failed to broadcast ChangeTurn: {}", e);
                                        }
                                    }
                                }
                            });

                            if game_ended {
                                let entries: Vec<shared::FinalScoreEntry> = {
                                    let room = gs.rooms.get(&room_id).unwrap();
                                    room.players
                                        .iter()
                                        .enumerate()
                                        .map(|(i, uuid)| {
                                            let player = gs.players.get(uuid).unwrap();
                                            shared::FinalScoreEntry {
                                                player: shared::PlayerHeader {
                                                    uuid: player.uuid,
                                                    name: player.name.clone(),
                                                },
                                                total: room
                                                    .scores
                                                    .get(i)
                                                    .map(|sheet| sheet.total())
                                                    .unwrap_or(0),
                                            }
                                        })
                                        .collect()
                                };
                                if let Some(room) = gs.rooms.get_mut(&room_id) {
                                    if let Err(e) = room
                                        .change_room
                                        .send(shared::RoomUpdateReason::FinalScores(entries))
                                    {
                                        log::warn!("Failed to broadcast FinalScores: {}", e);
                                    }
                                    if let Err(e) = room.change_room.send(
                                        shared::RoomUpdateReason::GameStateChange(
                                            GameState::Scoreboard,
                                        ),
                                    ) {
                                        log::warn!("Failed to broadcast GameStateChange: {}", e);
                                    }
                                }
                            }

                            if score_set {
                                ServerAPIResponse::Ok
                            } else {
                                ServerAPIResponse::Error(ErrorResponse::NotEnoughPermission)
                            }
                        } else {
                            ServerAPIResponse::Error(ErrorResponse::NotEnoughPermission)
                        }
                    } else {
                        ServerAPIResponse::Error(ErrorResponse::NotFound)
                    }
                } else {
                    ServerAPIResponse::Error(ErrorResponse::NotFound)
                }
            } else {
                ServerAPIResponse::Error(ErrorResponse::NotFound)
            }
        } else {
            ServerAPIResponse::Error(ErrorResponse::NotRegister)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{handle_request, GameState};
    use super::*;
    use crate::SocketLinkedData;
    use shared::{PlayerHeader, ScoreSheet, ServerAPICommand};
    use uuid::Uuid;

    fn register_cmd(uuid: Uuid, name: &str) -> ServerAPICommand {
        ServerAPICommand::Register(shared::RegisterCommand(
            shared::PROTOCOL_VERSION,
            uuid,
            name.to_string(),
        ))
    }

    async fn setup_started_room(
        game_state: &Arc<Mutex<GameState>>,
        socket_data: &mut SocketLinkedData,
    ) -> (Uuid, Uuid) {
        let creator_uuid = Uuid::new_v4();
        handle_request(&register_cmd(creator_uuid, "Alice"), game_state, socket_data).await;
        let res = handle_request(
            &ServerAPICommand::CreateRoom(shared::CreateRoomCommand("Room".to_string())),
            game_state,
            socket_data,
        )
        .await;
        let room_uuid = match res {
            ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(uuid)) => uuid,
            other => panic!("Response type not matching: {:?}", other),
        };
        let res = handle_request(
            &ServerAPICommand::StartGame(shared::StartGameCommand),
            game_state,
            socket_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Ok);
        (creator_uuid, room_uuid)
    }

    #[tokio::test]
    async fn test_select_points_guards() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let select_cmd =
            ServerAPICommand::SelectPoints(shared::SelectPointsCommand(shared::Score::Aces(0)));
        let mut socket_data = SocketLinkedData::default();

        let res = handle_request(&select_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotRegister)
        );

        let creator_uuid = Uuid::new_v4();
        let joiner_uuid = Uuid::new_v4();
        let mut creator_data = SocketLinkedData::default();
        let mut joiner_data = SocketLinkedData::default();

        handle_request(&register_cmd(creator_uuid, "Alice"), &game_state, &mut creator_data).await;
        let res = handle_request(
            &ServerAPICommand::CreateRoom(shared::CreateRoomCommand("Room".to_string())),
            &game_state,
            &mut creator_data,
        )
        .await;
        let room_uuid = match res {
            ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(uuid)) => uuid,
            other => panic!("Response type not matching: {:?}", other),
        };
        handle_request(&register_cmd(joiner_uuid, "Bob"), &game_state, &mut joiner_data).await;
        let res = handle_request(
            &ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)),
            &game_state,
            &mut joiner_data,
        )
        .await;
        assert!(matches!(
            res,
            ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(_))
        ));
        let res = handle_request(
            &ServerAPICommand::StartGame(shared::StartGameCommand),
            &game_state,
            &mut creator_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Ok);

        // Not the joiner's turn
        {
            let mut gs = game_state.lock().await;
            gs.rooms.get_mut(&room_uuid).unwrap().dices = [1, 2, 3, 4, 5];
        }
        let res = handle_request(&select_cmd, &game_state, &mut joiner_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotEnoughPermission)
        );

        // No dices rolled yet for creator turn
        {
            let mut gs = game_state.lock().await;
            gs.rooms.get_mut(&room_uuid).unwrap().dices = [0; 5];
        }
        let res = handle_request(&select_cmd, &game_state, &mut creator_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotEnoughPermission)
        );
        let _ = creator_uuid;
    }

    #[tokio::test]
    async fn test_select_points_full_game_ends_with_final_scores() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut socket_data = SocketLinkedData::default();
        let (creator_uuid, room_uuid) = setup_started_room(&game_state, &mut socket_data).await;

        let categories = [
            shared::Score::Aces(0),
            shared::Score::Twos(0),
            shared::Score::Threes(0),
            shared::Score::Fours(0),
            shared::Score::Fives(0),
            shared::Score::Sixes(0),
            shared::Score::ThreeOfAKind(0),
            shared::Score::FourOfAKind(0),
            shared::Score::Fullhouse(0),
            shared::Score::SmallStraight(0),
            shared::Score::LargeStraight(0),
            shared::Score::Yacht(0),
            shared::Score::Chance(0),
        ];

        for category in categories {
            {
                let mut gs = game_state.lock().await;
                gs.rooms.get_mut(&room_uuid).unwrap().dices = [1, 2, 3, 4, 5];
            }
            let cmd = ServerAPICommand::SelectPoints(shared::SelectPointsCommand(category));
            let res = handle_request(&cmd, &game_state, &mut socket_data).await;
            assert_eq!(res, ServerAPIResponse::Ok);
            // drain score/turn queues to avoid lag; keep room updates for final assertions
            while socket_data
                .listen_change_score
                .as_mut()
                .unwrap()
                .try_recv()
                .is_ok()
            {}
            while socket_data
                .listen_change_turn
                .as_mut()
                .unwrap()
                .try_recv()
                .is_ok()
            {}
        }

        let gs = game_state.lock().await;
        let room = gs.rooms.get(&room_uuid).unwrap();
        let sheet: &ScoreSheet = room.scores.first().unwrap();
        assert!(sheet.is_full());
        // dices [1,2,3,4,5]: upper = 15 (no bonus), small straight 30, large straight 40, chance 15
        assert_eq!(sheet.total(), 15 + 30 + 40 + 15);
        std::mem::drop(gs);

        let mut reasons = Vec::new();
        while let Ok(reason) = socket_data.listen_change_room.as_mut().unwrap().try_recv() {
            reasons.push(reason);
        }
        assert!(reasons.contains(&shared::RoomUpdateReason::GameStateChange(
            shared::GameState::Scoreboard
        )));
        let final_scores = reasons.iter().find_map(|r| match r {
            shared::RoomUpdateReason::FinalScores(entries) => Some(entries.clone()),
            _ => None,
        });
        let entries = final_scores.expect("FinalScores should be broadcast");
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].player,
            PlayerHeader {
                uuid: creator_uuid,
                name: "Alice".to_string()
            }
        );
        assert_eq!(entries[0].total, 15 + 30 + 40 + 15);
    }

    #[tokio::test]
    async fn test_select_yacht_grants_bonus_on_extra_yacht() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut socket_data = SocketLinkedData::default();
        let (_creator_uuid, room_uuid) = setup_started_room(&game_state, &mut socket_data).await;

        let yacht_cmd =
            ServerAPICommand::SelectPoints(shared::SelectPointsCommand(shared::Score::Yacht(0)));

        {
            let mut gs = game_state.lock().await;
            gs.rooms.get_mut(&room_uuid).unwrap().dices = [5, 5, 5, 5, 5];
        }
        let res = handle_request(&yacht_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Ok);

        {
            let mut gs = game_state.lock().await;
            gs.rooms.get_mut(&room_uuid).unwrap().dices = [5, 5, 5, 5, 5];
        }
        let res = handle_request(&yacht_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Ok);

        let gs = game_state.lock().await;
        let sheet = gs.rooms.get(&room_uuid).unwrap().scores.first().unwrap();
        assert_eq!(sheet.yacht, Some(50));
        assert_eq!(sheet.extra_yachts, 1);
        // yacht 50 + yacht bonus 50; no other category filled, upper subtotal 0 so no upper bonus
        assert_eq!(sheet.total(), 50 + 50);
    }

    #[tokio::test]
    async fn test_select_points_before_game_start_rejected() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let (mut creator_data, _joiner_data, _c, _j, room_uuid) =
            super::super::test_utils::setup_room(&game_state, false).await;
        {
            let mut gs = game_state.lock().await;
            gs.rooms.get_mut(&room_uuid).unwrap().dices = [1, 2, 3, 4, 5];
        }
        let res = handle_request(
            &ServerAPICommand::SelectPoints(shared::SelectPointsCommand(shared::Score::Aces(0))),
            &game_state,
            &mut creator_data,
        )
        .await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(ErrorResponse::NotEnoughPermission)
        );
    }

    #[tokio::test]
    async fn test_select_points_already_filled_category_rejected() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut socket_data = SocketLinkedData::default();
        let (_creator_uuid, room_uuid) = setup_started_room(&game_state, &mut socket_data).await;

        let aces_cmd =
            ServerAPICommand::SelectPoints(shared::SelectPointsCommand(shared::Score::Aces(0)));
        {
            let mut gs = game_state.lock().await;
            gs.rooms.get_mut(&room_uuid).unwrap().dices = [1, 2, 3, 4, 5];
        }
        let res = handle_request(&aces_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Ok);

        // Single player: the turn wraps back to 0. Selecting Aces again must
        // fail and leave the recorded score untouched.
        {
            let mut gs = game_state.lock().await;
            gs.rooms.get_mut(&room_uuid).unwrap().dices = [6, 6, 6, 6, 6];
        }
        let res = handle_request(&aces_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(ErrorResponse::NotEnoughPermission)
        );

        let gs = game_state.lock().await;
        let sheet = gs.rooms.get(&room_uuid).unwrap().scores.first().unwrap();
        assert_eq!(sheet.aces, Some(1)); // [1,2,3,4,5] scores a single ace
    }
}
