use super::Handler;
use shared::{
    count_points_for_fullhouse, count_points_for_identical, count_points_for_numbers,
    count_points_for_straight, ErrorResponse, GameState, SelectPointsCommand, ServerAPIResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct Score {
    aces: Option<u8>,
    twos: Option<u8>,
    threes: Option<u8>,
    fours: Option<u8>,
    fives: Option<u8>,
    sixes: Option<u8>,
    three_of_a_kind: Option<u8>,
    four_of_a_kind: Option<u8>,
    fullhouse: Option<u8>,
    small_straight: Option<u8>,
    large_straight: Option<u8>,
    yacht: Option<u8>,
    chance: Option<u8>,
}
impl Score {
    pub fn is_full(&self) -> bool {
        self.aces.is_some()
            && self.twos.is_some()
            && self.threes.is_some()
            && self.fours.is_some()
            && self.fives.is_some()
            && self.sixes.is_some()
            && self.three_of_a_kind.is_some()
            && self.four_of_a_kind.is_some()
            && self.fullhouse.is_some()
            && self.small_straight.is_some()
            && self.large_straight.is_some()
            && self.yacht.is_some()
            && self.chance.is_some()
    }
}

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

                        if gs.rooms.get(&room_id).unwrap().players[turn as usize] == *player_uuid {
                            let mut score_set = false;
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
                                            room.change_score
                                                .send((turn, shared::Score::Aces(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Twos(_) => {
                                        if current_score.twos.is_none() {
                                            let score = count_points_for_numbers(&room.dices, 2);
                                            current_score.twos = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::Twos(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Threes(_) => {
                                        if current_score.threes.is_none() {
                                            let score = count_points_for_numbers(&room.dices, 3);
                                            current_score.threes = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::Threes(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Fours(_) => {
                                        if current_score.fours.is_none() {
                                            let score = count_points_for_numbers(&room.dices, 4);
                                            current_score.fours = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::Fours(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Fives(_) => {
                                        if current_score.fives.is_none() {
                                            let score = count_points_for_numbers(&room.dices, 5);
                                            current_score.fives = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::Fives(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Sixes(_) => {
                                        if current_score.sixes.is_none() {
                                            let score = count_points_for_numbers(&room.dices, 6);
                                            current_score.sixes = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::Sixes(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::ThreeOfAKind(_) => {
                                        if current_score.three_of_a_kind.is_none() {
                                            let score = count_points_for_identical(&room.dices, 3);
                                            current_score.three_of_a_kind = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::ThreeOfAKind(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::FourOfAKind(_) => {
                                        if current_score.four_of_a_kind.is_none() {
                                            let score = count_points_for_identical(&room.dices, 4);
                                            current_score.four_of_a_kind = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::FourOfAKind(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Fullhouse(_) => {
                                        if current_score.fullhouse.is_none() {
                                            let score = count_points_for_fullhouse(&room.dices);
                                            current_score.fullhouse = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::Fullhouse(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::SmallStraight(_) => {
                                        if current_score.small_straight.is_none() {
                                            let score = count_points_for_straight(&room.dices, 4);
                                            current_score.small_straight = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::SmallStraight(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::LargeStraight(_) => {
                                        if current_score.large_straight.is_none() {
                                            let score = count_points_for_straight(&room.dices, 5);
                                            current_score.large_straight = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::LargeStraight(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Yacht(_) => {
                                        if current_score.yacht.is_none() {
                                            let score = count_points_for_identical(&room.dices, 5);
                                            current_score.yacht = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::Yacht(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                    shared::Score::Chance(_) => {
                                        if current_score.chance.is_none() {
                                            let score = count_points_for_identical(&room.dices, 1);
                                            current_score.chance = Some(score);
                                            room.change_score
                                                .send((turn, shared::Score::Chance(score)))
                                                .unwrap();
                                            score_set = true;
                                        }
                                    }
                                }

                                if score_set {
                                    room.throw_cnt = 0;
                                    room.turn = (room.turn + 1) % (room.players.len() as u8);
                                    if room.scores.iter().all(|score| score.is_full()) {
                                        room.change_room
                                            .send(shared::RoomUpdateReason::GameStateChange(
                                                GameState::Scoreboard,
                                            ))
                                            .unwrap();
                                    } else {
                                        room.dices = [0; 5];
                                        room.change_turn.send(room.turn as usize).unwrap();
                                    }
                                }
                            });

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
