use super::Handler;
use shared::{ErrorResponse, GameState, SelectPointsCommand, ServerAPIResponse};
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
                                            let score = count_points_for_numbers(&room.dices, 4);
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
                                            let score = count_points_for_numbers(&room.dices, 5);
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
                                        room.change_game_state.send(GameState::Scoreboard).unwrap();
                                    } else {
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

fn count_points_for_numbers(dices: &[u8; 5], number: u8) -> u8 {
    let mut points = 0;
    for dice in dices.iter() {
        if *dice == number {
            points += *dice;
        }
    }

    points
}

fn count_points_for_identical(dices: &[u8; 5], number: u8) -> u8 {
    let mut value_freq = [0; 6];

    for dice in dices {
        value_freq[(*dice - 1) as usize] += 1;
    }

    if *value_freq.iter().max().unwrap() >= number {
        if number == 5 {
            50
        } else {
            dices.iter().sum()
        }
    } else {
        0
    }
}

fn count_points_for_fullhouse(dices: &[u8; 5]) -> u8 {
    let mut value_freq = [0; 6];

    for dice in dices {
        value_freq[(*dice - 1) as usize] += 1;
    }

    let freq_max = *value_freq.iter().max().unwrap();
    let freq_min = *value_freq.iter().filter(|x| **x > 0).min().unwrap();

    if freq_max == 5 || (freq_max == 3 && freq_min == 2) {
        25
    } else {
        0
    }
}

fn count_points_for_straight(dices: &[u8; 5], size: u8) -> u8 {
    let mut value_straight_size = [0; 5];
    let mut dices_sorted = *dices;
    dices_sorted.sort();

    for (index, dice) in dices_sorted.iter().enumerate() {
        let mut prev_val = *dice;
        let mut counter = 1;
        for next_dice_sort in dices_sorted.iter().skip(index + 1) {
            if (prev_val + 1) != *next_dice_sort {
                break;
            }
            prev_val = *next_dice_sort;
            counter += 1;
        }

        value_straight_size[index] = counter;
    }

    if *value_straight_size.iter().max().unwrap() >= size {
        if size == 5 {
            40
        } else {
            30
        }
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::super::{handle_request, GameState};
    use super::*;
    use crate::SocketLinkedData;
    use shared::ServerAPICommand;

    #[test]
    fn test_number_counter() {
        assert_eq!(count_points_for_numbers(&[1, 2, 5, 6, 2], 1), 1);
        assert_eq!(count_points_for_numbers(&[1, 2, 5, 6, 2], 2), 4);
        assert_eq!(count_points_for_numbers(&[1, 2, 5, 6, 2], 3), 0);
        assert_eq!(count_points_for_numbers(&[4, 2, 4, 4, 4], 4), 16);
        assert_eq!(count_points_for_numbers(&[5, 5, 5, 5, 5], 5), 25);
        assert_eq!(count_points_for_numbers(&[6, 6, 5, 6, 2], 6), 18);
    }

    #[tokio::test]
    async fn test_points_for_identical() {
        assert_eq!(count_points_for_identical(&[1, 2, 5, 6, 2], 1), 16);

        assert_eq!(count_points_for_identical(&[1, 2, 5, 6, 2], 3), 0);
        assert_eq!(count_points_for_identical(&[5, 5, 5, 5, 5], 3), 25);
        assert_eq!(count_points_for_identical(&[5, 5, 5, 5, 1], 3), 21);
        assert_eq!(count_points_for_identical(&[4, 4, 4, 6, 1], 3), 19);

        assert_eq!(count_points_for_identical(&[1, 2, 5, 6, 2], 4), 0);
        assert_eq!(count_points_for_identical(&[5, 5, 5, 5, 5], 4), 25);
        assert_eq!(count_points_for_identical(&[5, 5, 5, 5, 1], 4), 21);

        assert_eq!(count_points_for_identical(&[1, 2, 5, 6, 2], 5), 0);
        assert_eq!(count_points_for_identical(&[5, 5, 5, 5, 5], 5), 50);
    }

    #[tokio::test]
    async fn test_points_for_fullhouse() {
        assert_eq!(count_points_for_fullhouse(&[2, 2, 6, 6, 6]), 25);
        assert_eq!(count_points_for_fullhouse(&[1, 1, 1, 1, 1]), 25);
        assert_eq!(count_points_for_fullhouse(&[5, 5, 5, 5, 1]), 0);
        assert_eq!(count_points_for_fullhouse(&[1, 2, 5, 6, 2]), 0);
    }

    #[tokio::test]
    async fn test_points_for_straight() {
        assert_eq!(count_points_for_straight(&[2, 3, 4, 5, 6], 4), 30);
        assert_eq!(count_points_for_straight(&[1, 3, 4, 5, 6], 4), 30);
        assert_eq!(count_points_for_straight(&[1, 3, 4, 3, 6], 4), 0);

        assert_eq!(count_points_for_straight(&[2, 3, 4, 5, 6], 5), 40);
        assert_eq!(count_points_for_straight(&[1, 3, 4, 5, 6], 5), 0);
    }
}
