use bevy::prelude::*;
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
enum ScoreSelector {
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
                            Button,
                            Node {
                                padding: UiRect::all(Val::Px(5.0)),
                                margin: UiRect::all(Val::Px(10.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
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
