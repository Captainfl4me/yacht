use std::fmt::Display;

use bevy::prelude::*;
use shared::{
    count_points_for_fullhouse, count_points_for_identical, count_points_for_numbers,
    count_points_for_straight,
};
use strum_macros::EnumIter;
use uuid::Uuid;

use crate::{
    colors::{
        ACCENT, BUTTON_BG, HOVERED_PRESSED_BUTTON, INK, NORMAL_BUTTON, PANEL_COLOR, PRIMARY,
    },
    network::{
        events::{ChangeScoreEvent, RollEvent},
        NetworkCommandEvent,
    },
    style, ButtonDisable, PlayerData, PrimaryButton,
};
use bevy_persistent::Persistent;
use shared::ScoreSheet;

use super::{CurrentTurn, RoomPlayers, ThrowsUsed};

#[derive(Component)]
pub enum GameButtonAction {
    Throw,
}

#[derive(Component)]
pub struct TopLevelGameScreen;

#[derive(Component)]
pub struct PlayersList;

#[derive(Component)]
pub struct PlayerUuid(pub Uuid);

#[derive(Component)]
pub struct DicesList;

#[derive(Component)]
pub struct DicesSaved;

#[derive(Component)]
pub struct TurnIndicator;

#[derive(Component)]
pub struct PlayerChip;

#[derive(Component)]
pub struct DiceSpawnAnim(pub Timer);

#[derive(Component)]
pub struct ThrowsLeftText;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TotalsKind {
    Upper,
    Bonus,
    Total,
}

#[derive(Component)]
pub struct TotalsText(pub TotalsKind);

#[derive(Resource, Default)]
pub struct MyScoreSheet(pub ScoreSheet);

#[derive(Component)]
pub struct DiceIndex(usize);

#[derive(Component)]
pub struct ScorePlayable;

#[derive(Resource, Default)]
pub struct DiceMask([u8; 5]);

#[derive(Component, EnumIter, Copy, Clone, PartialEq, Eq, Debug)]
pub enum ScoreSelector {
    Aces,
    Twos,
    Threes,
    Fours,
    Fives,
    Sixes,
    ThreeOfAKind,
    FourOfAKind,
    Fullhouse,
    SmallStraight,
    LargeStraight,
    Yacht,
    Chance,
}
impl Display for ScoreSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ScoreSelector::Aces => "1",
                ScoreSelector::Twos => "2",
                ScoreSelector::Threes => "3",
                ScoreSelector::Fours => "4",
                ScoreSelector::Fives => "5",
                ScoreSelector::Sixes => "6",
                ScoreSelector::ThreeOfAKind => "3x",
                ScoreSelector::FourOfAKind => "4x",
                ScoreSelector::Fullhouse => "full",
                ScoreSelector::SmallStraight => "ss",
                ScoreSelector::LargeStraight => "ls",
                ScoreSelector::Yacht => "5x",
                ScoreSelector::Chance => "?",
            }
        )
    }
}

impl ScoreSelector {
    pub fn from_shared(score: &shared::Score) -> ScoreSelector {
        match score {
            shared::Score::Aces(_) => ScoreSelector::Aces,
            shared::Score::Twos(_) => ScoreSelector::Twos,
            shared::Score::Threes(_) => ScoreSelector::Threes,
            shared::Score::Fours(_) => ScoreSelector::Fours,
            shared::Score::Fives(_) => ScoreSelector::Fives,
            shared::Score::Sixes(_) => ScoreSelector::Sixes,
            shared::Score::ThreeOfAKind(_) => ScoreSelector::ThreeOfAKind,
            shared::Score::FourOfAKind(_) => ScoreSelector::FourOfAKind,
            shared::Score::Fullhouse(_) => ScoreSelector::Fullhouse,
            shared::Score::SmallStraight(_) => ScoreSelector::SmallStraight,
            shared::Score::LargeStraight(_) => ScoreSelector::LargeStraight,
            shared::Score::Yacht(_) => ScoreSelector::Yacht,
            shared::Score::Chance(_) => ScoreSelector::Chance,
        }
    }
}

pub fn shared_score_value(score: &shared::Score) -> u8 {
    match score {
        shared::Score::Aces(v) => *v,
        shared::Score::Twos(v) => *v,
        shared::Score::Threes(v) => *v,
        shared::Score::Fours(v) => *v,
        shared::Score::Fives(v) => *v,
        shared::Score::Sixes(v) => *v,
        shared::Score::ThreeOfAKind(v) => *v,
        shared::Score::FourOfAKind(v) => *v,
        shared::Score::Fullhouse(v) => *v,
        shared::Score::SmallStraight(v) => *v,
        shared::Score::LargeStraight(v) => *v,
        shared::Score::Yacht(v) => *v,
        shared::Score::Chance(v) => *v,
    }
}

pub fn my_player_index(room_players: &RoomPlayers, uuid: Uuid) -> Option<usize> {
    room_players.0.iter().position(|p| p.uuid == uuid)
}

pub fn round_setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Stretch,
                ..default()
            },
            TopLevelGameScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        width: Val::Px(320.0),
                        padding: UiRect::all(Val::Px(18.0)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(PANEL_COLOR),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        style::label_ink("SCORE SHEET", 18.0),
                        Node {
                            margin: UiRect::bottom(Val::Px(2.0)),
                            ..default()
                        },
                    ));
                    parent.spawn((
                        style::label_ink_dim("fill every line to win", 11.0),
                        Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    parent.spawn(section_header("UPPER SECTION"));
                    for score in [
                        ScoreSelector::Aces,
                        ScoreSelector::Twos,
                        ScoreSelector::Threes,
                        ScoreSelector::Fours,
                        ScoreSelector::Fives,
                        ScoreSelector::Sixes,
                    ] {
                        parent.spawn(score_row(score));
                    }
                    parent.spawn(totals_row("subtotal", TotalsKind::Upper, false));
                    parent.spawn(totals_row("bonus ≥ 63", TotalsKind::Bonus, false));
                    parent.spawn(divider());

                    parent.spawn(section_header("LOWER SECTION"));
                    for score in [
                        ScoreSelector::ThreeOfAKind,
                        ScoreSelector::FourOfAKind,
                        ScoreSelector::Fullhouse,
                        ScoreSelector::SmallStraight,
                        ScoreSelector::LargeStraight,
                        ScoreSelector::Yacht,
                        ScoreSelector::Chance,
                    ] {
                        parent.spawn(score_row(score));
                    }
                    parent.spawn(divider());
                    parent.spawn(totals_row("TOTAL", TotalsKind::Total, true));
                });

            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Stretch,
                    padding: UiRect::all(Val::Px(24.0)),
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        style::label("TURN: -", 18.0),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                        TurnIndicator,
                    ));

                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        PlayersList,
                    ));

                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::vertical(Val::Px(16.0)),
                            ..default()
                        },
                        DicesList,
                    ));

                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            let mut throw_button = parent.spawn((
                                Button,
                                Node {
                                    height: Val::Px(52.0),
                                    padding: UiRect::horizontal(Val::Px(28.0)),
                                    margin: UiRect::vertical(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(NORMAL_BUTTON),
                                GameButtonAction::Throw,
                            ));
                            throw_button.insert(PrimaryButton);
                            throw_button.with_children(|parent| {
                                parent.spawn(style::label_on_primary("THROW", 18.0));
                            });

                            parent.spawn((
                                style::label_dim(
                                    format!("THROWS LEFT: {}", shared::MAX_THROWS_PER_TURN),
                                    12.0,
                                ),
                                ThrowsLeftText,
                            ));

                            parent.spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                DicesSaved,
                            ));
                        });
                });
        });

    commands.insert_resource(DiceMask::default());
}

fn section_header(text: &'static str) -> impl Bundle + use<> {
    (
        style::label_ink_dim(text, 11.0),
        Node {
            margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(6.0), Val::Px(3.0)),
            ..default()
        },
    )
}

fn score_row(score: ScoreSelector) -> impl Bundle + use<> {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::vertical(Val::Px(2.0)),
            ..default()
        },
        children![
            style::label_ink(format!("{score}"), 14.0),
            (
                Button,
                Node {
                    width: Val::Px(58.0),
                    height: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_BG),
                BorderColor(INK),
                style::radius_sm(),
                score,
                ScorePlayable,
                ButtonDisable,
                children![style::label_ink("0", 14.0)],
            )
        ],
    )
}

fn totals_row(label: &'static str, kind: TotalsKind, emphasize: bool) -> impl Bundle + use<> {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::vertical(Val::Px(3.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        if emphasize {
            BackgroundColor(HOVERED_PRESSED_BUTTON)
        } else {
            BackgroundColor(Color::NONE)
        },
        if emphasize {
            BorderColor(INK)
        } else {
            BorderColor(Color::NONE)
        },
        style::radius_sm(),
        children![
            if emphasize {
                style::label_ink(label, 14.0)
            } else {
                style::label_ink_dim(label, 12.0)
            },
            (
                if emphasize {
                    style::label_ink("0", 14.0)
                } else {
                    style::label_ink("0", 12.0)
                },
                TotalsText(kind),
            ),
        ],
    )
}

fn divider() -> impl Bundle + use<> {
    (
        Node {
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(crate::colors::TEXT_INK_DIM),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn dices_throw_update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut roll_event: EventReader<RollEvent>,
    query_dices_list: Query<Entity, With<DicesList>>,
    query_score_button: Query<(Entity, &ScoreSelector), With<ScorePlayable>>,
    dice_on_board_query: Query<Entity, (With<Button>, With<DiceIndex>)>,
    mut query_text: Query<&mut Text>,
    children_query: Query<&Children>,
    dice_mask: Res<DiceMask>,
) {
    if let Some(RollEvent(shared::RollResponse(dices))) = roll_event.read().next() {
        reset_center_dice(
            &mut commands,
            query_dices_list,
            children_query,
            dice_on_board_query,
        );

        if let Ok(dices_list) = query_dices_list.single() {
            commands.entity(dices_list).with_children(|parent| {
                for (index, dice) in dices.iter().enumerate() {
                    if dice_mask.0[index] == 0 {
                        parent.spawn((
                            Node {
                                width: Val::Px(84.0),
                                margin: UiRect::all(Val::Px(8.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ImageNode::new(asset_server.load(format!("dice_{dice}.png"))),
                            Button,
                            DiceIndex(index),
                            DiceSpawnAnim(Timer::from_seconds(0.15, TimerMode::Once)),
                        ));
                    }
                }
            });

            for (score_button_entity, score_type) in query_score_button {
                commands
                    .entity(score_button_entity)
                    .remove::<ButtonDisable>();

                let score = match score_type {
                    ScoreSelector::Aces => count_points_for_numbers(dices, 1),
                    ScoreSelector::Twos => count_points_for_numbers(dices, 2),
                    ScoreSelector::Threes => count_points_for_numbers(dices, 3),
                    ScoreSelector::Fours => count_points_for_numbers(dices, 4),
                    ScoreSelector::Fives => count_points_for_numbers(dices, 5),
                    ScoreSelector::Sixes => count_points_for_numbers(dices, 6),
                    ScoreSelector::ThreeOfAKind => count_points_for_identical(dices, 3),
                    ScoreSelector::FourOfAKind => count_points_for_identical(dices, 4),
                    ScoreSelector::Fullhouse => count_points_for_fullhouse(dices),
                    ScoreSelector::SmallStraight => count_points_for_straight(dices, 4),
                    ScoreSelector::LargeStraight => count_points_for_straight(dices, 5),
                    ScoreSelector::Yacht => count_points_for_identical(dices, 5),
                    ScoreSelector::Chance => count_points_for_identical(dices, 1),
                };
                if let Ok(children) = children_query.get(score_button_entity) {
                    if let Ok(text) = &mut query_text.get_mut(children.iter().next().unwrap()) {
                        **text = Text::new(format!("{score}"));
                    }
                }
            }
        }
    }
}

type GameActionInteractionQueryType<'a, 'b, 'c> = Query<
    'c,
    'b,
    (&'a Interaction, &'a GameButtonAction),
    (Changed<Interaction>, With<Button>, Without<ButtonDisable>),
>;

pub fn button_action(
    interaction_query: GameActionInteractionQueryType,
    mut network_command: EventWriter<NetworkCommandEvent>,
    mut throws_used: ResMut<ThrowsUsed>,
) {
    for (interaction, button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button_action {
                GameButtonAction::Throw => {
                    throws_used.0 = (throws_used.0 + 1).min(shared::MAX_THROWS_PER_TURN);
                    network_command.write(NetworkCommandEvent(shared::ServerAPICommand::Roll(
                        shared::RollCommand,
                    )));
                }
            }
        }
    }
}

type ScoreInteractionQueryType<'a, 'b, 'c> = Query<
    'c,
    'b,
    (Entity, &'a Interaction, &'a ScoreSelector),
    (Changed<Interaction>, With<Button>, Without<ButtonDisable>),
>;
#[allow(clippy::too_many_arguments)]
pub fn click_on_score(
    mut commands: Commands,
    interaction_query: ScoreInteractionQueryType,
    query_all_score_button: Query<Entity, With<ScorePlayable>>,
    mut query_text: Query<&mut Text>,
    mut dice_mask: ResMut<DiceMask>,
    children_query: Query<&Children>,
    query_dices_list: Query<Entity, With<DicesList>>,
    query_dices_saved_list: Query<Entity, With<DicesSaved>>,
    dice_on_board_query: Query<Entity, (With<Button>, With<DiceIndex>)>,
    mut network_command: EventWriter<NetworkCommandEvent>,
) {
    for (selected_button_entity, interaction, score_selector) in &interaction_query {
        if *interaction == Interaction::Pressed {
            network_command.write(NetworkCommandEvent(shared::ServerAPICommand::SelectPoints(
                shared::SelectPointsCommand(match score_selector {
                    ScoreSelector::Aces => shared::Score::Aces(0),
                    ScoreSelector::Twos => shared::Score::Twos(0),
                    ScoreSelector::Threes => shared::Score::Threes(0),
                    ScoreSelector::Fours => shared::Score::Fours(0),
                    ScoreSelector::Fives => shared::Score::Fives(0),
                    ScoreSelector::Sixes => shared::Score::Sixes(0),
                    ScoreSelector::ThreeOfAKind => shared::Score::ThreeOfAKind(0),
                    ScoreSelector::FourOfAKind => shared::Score::FourOfAKind(0),
                    ScoreSelector::Fullhouse => shared::Score::Fullhouse(0),
                    ScoreSelector::SmallStraight => shared::Score::SmallStraight(0),
                    ScoreSelector::LargeStraight => shared::Score::LargeStraight(0),
                    ScoreSelector::Yacht => shared::Score::Yacht(0),
                    ScoreSelector::Chance => shared::Score::Chance(0),
                }),
            )));

            for button_entity in query_all_score_button {
                commands.entity(button_entity).insert(ButtonDisable);

                if button_entity == selected_button_entity {
                    continue;
                }

                if let Ok(children) = children_query.get(button_entity) {
                    if let Ok(text) = &mut query_text.get_mut(children.iter().next().unwrap()) {
                        **text = Text::new("0".to_string());
                    }
                }
            }

            commands
                .entity(selected_button_entity)
                .remove::<ScorePlayable>();
            
            dice_mask.0 = [0; 5];

            reset_center_dice(
                &mut commands,
                query_dices_list,
                children_query,
                dice_on_board_query,
            );

            reset_save_dice(
                &mut commands,
                query_dices_saved_list,
                children_query,
                dice_on_board_query,
            );
        }
    }
}

type DiceInteractionQueryType<'a, 'b, 'c> = Query<
    'c,
    'b,
    (Entity, &'a Interaction, &'a DiceIndex, &'a ChildOf),
    (Changed<Interaction>, With<Button>, Without<ButtonDisable>),
>;
pub fn click_on_dice(
    mut commands: Commands,
    interaction_query: DiceInteractionQueryType,
    query_dices_saved_list: Query<Entity, With<DicesSaved>>,
    query_dices_list: Query<Entity, With<DicesList>>,
    mut dice_mask: ResMut<DiceMask>,
    mut image_query: Query<&mut ImageNode>,
    mut network_command: EventWriter<NetworkCommandEvent>,
) {
    for (selected_dice_entity, interaction, dice_index, parent) in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Ok(dices_saved_list) = query_dices_saved_list.single() {
                dice_mask.0[dice_index.0] = if dice_mask.0[dice_index.0] == 0 { 1 } else { 0 };
                let saved = dice_mask.0[dice_index.0] == 1;
                if let Ok(mut image) = image_query.get_mut(selected_dice_entity) {
                    image.color = if saved { ACCENT } else { Color::WHITE };
                }

                if parent.0 == dices_saved_list {
                    if let Ok(dices_list) = query_dices_list.single() {
                        commands
                            .entity(selected_dice_entity)
                            .insert(ChildOf(dices_list));
                    }
                } else {
                    commands
                        .entity(selected_dice_entity)
                        .insert(ChildOf(dices_saved_list));
                }

                network_command.write(NetworkCommandEvent(shared::ServerAPICommand::KeepDice(
                    shared::KeepDiceCommand(dice_mask.0),
                )));
            }
        }
    }
}

fn reset_center_dice(
    commands: &mut Commands,
    query_dices_list: Query<Entity, With<DicesList>>,
    children_query: Query<&Children>,
    dice_on_board_query: Query<Entity, (With<Button>, With<DiceIndex>)>,
) {
    if let Ok(dices_list) = query_dices_list.single() {
        if let Ok(children) = children_query.get(dices_list) {
            for dice in dice_on_board_query.iter_many(children) {
                commands.entity(dice).despawn();
            }
        }
    }
}

fn reset_save_dice(
    commands: &mut Commands,
    query_dices_saved_list: Query<Entity, With<DicesSaved>>,
    children_query: Query<&Children>,
    dice_on_board_query: Query<Entity, (With<Button>, With<DiceIndex>)>,
) {
    if let Ok(dices_list) = query_dices_saved_list.single() {
        if let Ok(children) = children_query.get(dices_list) {
            for dice in dice_on_board_query.iter_many(children) {
                commands.entity(dice).despawn();
            }
        }
    }
}

pub fn turn_indicator_update(
    current_turn: Res<CurrentTurn>,
    room_players: Res<RoomPlayers>,
    player_data: Res<Persistent<PlayerData>>,
    mut query: Query<&mut Text, With<TurnIndicator>>,
) {
    if !current_turn.is_changed() && !room_players.is_changed() {
        return;
    }
    for mut text in &mut query {
        let name = room_players
            .0
            .get(current_turn.0)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "?".to_string());
        let mine = my_player_index(&room_players, player_data.uuid) == Some(current_turn.0);
        *text = Text::new(format!(
            "Turn: {}{}",
            name,
            if mine { " (you)" } else { "" }
        ));
    }
}

pub fn players_list_update(
    mut commands: Commands,
    room_players: Res<RoomPlayers>,
    current_turn: Res<CurrentTurn>,
    query_list: Query<Entity, With<PlayersList>>,
    children_query: Query<&Children>,
) {
    if !room_players.is_changed() && !current_turn.is_changed() {
        return;
    }
    if let Ok(list) = query_list.single() {
        if let Ok(children) = children_query.get(list) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        for (i, player) in room_players.0.iter().enumerate() {
            let highlight = i == current_turn.0;
            commands.entity(list).with_children(|parent| {
                parent.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(6.0)),
                        margin: UiRect::horizontal(Val::Px(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(if highlight {
                        HOVERED_PRESSED_BUTTON
                    } else {
                        BUTTON_BG
                    }),
                    BorderColor(INK),
                    style::radius_sm(),
                    PlayerChip,
                    children![style::label_ink(player.name.clone(), 14.0)],
                ));
            });
        }
    }
}

pub fn throw_button_gate(
    mut commands: Commands,
    current_turn: Res<CurrentTurn>,
    room_players: Res<RoomPlayers>,
    throws_used: Res<ThrowsUsed>,
    player_data: Res<Persistent<PlayerData>>,
    query_throw: Query<Entity, (With<Button>, With<GameButtonAction>)>,
    query_dice: Query<Entity, (With<Button>, With<DiceIndex>, Without<GameButtonAction>)>,
) {
    if !current_turn.is_changed() && !room_players.is_changed() && !throws_used.is_changed() {
        return;
    }
    let my_turn = my_player_index(&room_players, player_data.uuid) == Some(current_turn.0);
    let throws_left = throws_used.0 < shared::MAX_THROWS_PER_TURN;
    for entity in &query_throw {
        if my_turn && throws_left {
            commands.entity(entity).remove::<ButtonDisable>();
        } else {
            commands.entity(entity).insert(ButtonDisable);
        }
    }
    for entity in &query_dice {
        if my_turn {
            commands.entity(entity).remove::<ButtonDisable>();
        } else {
            commands.entity(entity).insert(ButtonDisable);
        }
    }
}

pub fn score_change_update(
    mut change_score_event: EventReader<ChangeScoreEvent>,
    room_players: Res<RoomPlayers>,
    player_data: Res<Persistent<PlayerData>>,
    mut my_score_sheet: ResMut<MyScoreSheet>,
    query_score_button: Query<(Entity, &ScoreSelector), With<Button>>,
    mut query_text: Query<&mut Text>,
    mut query_color: Query<&mut TextColor>,
    children_query: Query<&Children>,
) {
    for ChangeScoreEvent(shared::ChangeScoreResponse(player_index, score)) in
        change_score_event.read()
    {
        let mine = my_player_index(&room_players, player_data.uuid) == Some(*player_index as usize);
        if !mine {
            continue;
        }
        my_score_sheet.0.set(*score);
        let selector = ScoreSelector::from_shared(score);
        let value = shared_score_value(score);
        for (entity, sel) in &query_score_button {
            if *sel == selector {
                if let Ok(children) = children_query.get(entity) {
                    if let Some(child) = children.iter().next() {
                        if let Ok(mut text) = query_text.get_mut(child) {
                            *text = Text::new(format!("{value}"));
                        }
                        if let Ok(mut color) = query_color.get_mut(child) {
                            *color = TextColor(PRIMARY);
                        }
                    }
                }
            }
        }
    }
}

pub fn dice_spawn_anim(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DiceSpawnAnim, &mut Transform)>,
) {
    for (entity, mut anim, mut transform) in &mut query {
        anim.0.tick(time.delta());
        if anim.0.finished() {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<DiceSpawnAnim>();
        } else {
            let t = anim.0.fraction();
            transform.scale = Vec3::splat(0.5 + 0.5 * t);
        }
    }
}

pub fn score_sheet_totals_update(
    sheet: Res<MyScoreSheet>,
    mut query: Query<(&TotalsText, &mut Text)>,
) {
    if !sheet.is_changed() {
        return;
    }
    for (kind, mut text) in &mut query {
        let value = match kind.0 {
            TotalsKind::Upper => sheet.0.upper_subtotal(),
            TotalsKind::Bonus => {
                sheet.0.upper_bonus() + shared::YACHT_BONUS * sheet.0.extra_yachts as u16
            }
            TotalsKind::Total => sheet.0.total(),
        };
        *text = Text::new(format!("{value}"));
    }
}

pub fn throws_left_update(
    throws_used: Res<ThrowsUsed>,
    mut query: Query<&mut Text, With<ThrowsLeftText>>,
) {
    if !throws_used.is_changed() {
        return;
    }
    for mut text in &mut query {
        *text = Text::new(format!(
            "Throws left: {}",
            shared::MAX_THROWS_PER_TURN.saturating_sub(throws_used.0)
        ));
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_selector_from_shared() {
        assert_eq!(
            ScoreSelector::from_shared(&shared::Score::Aces(3)),
            ScoreSelector::Aces
        );
        assert_eq!(
            ScoreSelector::from_shared(&shared::Score::Yacht(50)),
            ScoreSelector::Yacht
        );
        assert_eq!(
            ScoreSelector::from_shared(&shared::Score::Chance(12)),
            ScoreSelector::Chance
        );
        assert_eq!(shared_score_value(&shared::Score::Fullhouse(25)), 25);
        assert_eq!(shared_score_value(&shared::Score::LargeStraight(40)), 40);
        assert_eq!(shared_score_value(&shared::Score::SmallStraight(0)), 0);
    }

    #[test]
    fn test_my_player_index() {
        let players = RoomPlayers(vec![
            shared::PlayerHeader {
                uuid: Uuid::new_v4(),
                name: "A".to_string(),
            },
            shared::PlayerHeader {
                uuid: Uuid::new_v4(),
                name: "B".to_string(),
            },
        ]);
        assert_eq!(my_player_index(&players, players.0[1].uuid), Some(1));
        assert_eq!(my_player_index(&players, players.0[0].uuid), Some(0));
        assert_eq!(my_player_index(&players, Uuid::new_v4()), None);
    }
}
