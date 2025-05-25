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
pub struct DiceIndex(usize);

#[derive(Component, EnumIter)]
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
                        });
                });
        });
}

pub fn dices_throw_update(
    mut commands: Commands,
    mut roll_event: EventReader<RollEvent>,
    query_dices_list: Query<Entity, With<DicesList>>,
    query_score_button: Query<(Entity, &ScoreSelector)>,
    mut query_text: Query<&mut Text>,
    children_query: Query<&Children>,
) {
    if let Some(RollEvent(shared::RollResponse(dices))) = roll_event.read().next() {
        if let Ok(dices_list) = query_dices_list.single() {
            commands.entity(dices_list).with_children(|parent| {
                for (index, dice) in dices.iter().enumerate() {
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
            });

            for (score_button_entity, score_type) in query_score_button {
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

type GameActionInteractionQueryType<'a, 'b, 'c> =
    Query<'c, 'b, (&'a Interaction, &'a GameButtonAction), (Changed<Interaction>, With<Button>)>;

pub fn button_action(
    mut commands: Commands,
    interaction_query: GameActionInteractionQueryType,
    dice_on_board_query: Query<Entity, (With<Button>, With<DiceIndex>)>,
    children_query: Query<&Children>,
    query_dices_list: Query<Entity, With<DicesList>>,
    mut network_command: EventWriter<NetworkCommandEvent>,
) {
    for (interaction, button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button_action {
                GameButtonAction::Throw => {
                    network_command.write(NetworkCommandEvent(shared::ServerAPICommand::Roll(
                        shared::RollCommand,
                    )));

                    if let Ok(dices_list) = query_dices_list.single() {
                        if let Ok(children) = children_query.get(dices_list) {
                            for dice in dice_on_board_query.iter_many(children) {
                                commands.entity(dice).despawn();
                            }
                        }
                    }
                }
            }
        }
    }
}
