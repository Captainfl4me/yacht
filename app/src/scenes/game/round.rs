use std::fmt::Display;

use bevy::prelude::*;
use shared::{
    count_points_for_fullhouse, count_points_for_identical, count_points_for_numbers,
    count_points_for_straight,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use uuid::Uuid;

use crate::{
    colors::{BACKGROUND_COLOR, NORMAL_BUTTON, TEXT_COLOR},
    network::{events::RollEvent, NetworkCommandEvent},
    ButtonDisable,
};

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
pub struct DiceIndex(usize);

#[derive(Component)]
pub struct ScorePlayable;

#[derive(Resource, Default)]
pub struct DiceMask([u8; 5]);

#[derive(Component, EnumIter, Copy, Clone)]
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
                        width: Val::Percent(20.0),
                        ..default()
                    },
                    BackgroundColor(BACKGROUND_COLOR),
                ))
                .with_children(|parent| {
                    for score in ScoreSelector::iter() {
                        parent.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Stretch,
                                ..default()
                            },
                            children![
                                (
                                    Text::new(format!("{score}")),
                                    TextFont {
                                        font_size: 33.0,
                                        ..default()
                                    },
                                    TextColor(TEXT_COLOR),
                                ),
                                (
                                    Button,
                                    Node {
                                        padding: UiRect::all(Val::Px(5.0)),
                                        margin: UiRect::all(Val::Px(10.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        width: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(NORMAL_BUTTON),
                                    score,
                                    ScorePlayable,
                                    ButtonDisable,
                                    children![(
                                        Text::new("0"),
                                        TextFont {
                                            font_size: 33.0,
                                            ..default()
                                        },
                                        TextColor(TEXT_COLOR),
                                    )],
                                )
                            ],
                        ));
                    }
                });

            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Stretch,
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Stretch,
                            ..default()
                        },
                        PlayersList,
                    ));

                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        DicesList,
                    ));

                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Stretch,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Button,
                                Node {
                                    padding: UiRect::all(Val::Px(5.0)),
                                    margin: UiRect::all(Val::Px(10.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(NORMAL_BUTTON),
                                GameButtonAction::Throw,
                                children![(
                                    Text::new("Throw"),
                                    TextFont {
                                        font_size: 33.0,
                                        ..default()
                                    },
                                    TextColor(TEXT_COLOR),
                                )],
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

#[allow(clippy::too_many_arguments)]
pub fn dices_throw_update(
    mut commands: Commands,
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
                                padding: UiRect::all(Val::Px(5.0)),
                                margin: UiRect::all(Val::Px(10.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            Button,
                            DiceIndex(index),
                            children![(
                                Text::new(format!("{dice}")),
                                TextFont {
                                    font_size: 33.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            )],
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
) {
    for (interaction, button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button_action {
                GameButtonAction::Throw => {
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
pub fn click_on_score(
    mut commands: Commands,
    interaction_query: ScoreInteractionQueryType,
    query_all_score_button: Query<Entity, With<ScorePlayable>>,
    mut query_text: Query<&mut Text>,
    children_query: Query<&Children>,
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
    mut network_command: EventWriter<NetworkCommandEvent>,
) {
    for (selected_dice_entity, interaction, dice_index, parent) in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Ok(dices_saved_list) = query_dices_saved_list.single() {
                dice_mask.0[dice_index.0] = if dice_mask.0[dice_index.0] == 0 { 1 } else { 0 };

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
