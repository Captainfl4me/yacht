use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::{
    colors::{BACKGROUND_COLOR, HOVERED_PRESSED_BUTTON, NORMAL_BUTTON, TEXT_COLOR},
    network::NetworkCommandEvent,
    ButtonDisable, PlayerData,
};

use super::{FinalScores, RoomCreator};

#[derive(Component)]
pub struct TopLevelScoreboardScreen;

#[derive(Component)]
pub struct ScoreboardList;

#[derive(Component)]
pub struct ScoreboardTitle;

#[derive(Component)]
pub enum ScoreboardButtonAction {
    PlayAgain,
    LeaveRoom,
}

pub fn scoreboard_setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            TopLevelScoreboardScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    BackgroundColor(BACKGROUND_COLOR),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Scoreboard"),
                        TextFont {
                            font_size: 67.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                        ScoreboardTitle,
                    ));

                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            ..default()
                        },
                        ScoreboardList,
                    ));

                    parent.spawn((
                        Button,
                        Node {
                            height: Val::Px(65.0),
                            margin: UiRect::all(Val::Px(20.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(NORMAL_BUTTON),
                        ScoreboardButtonAction::PlayAgain,
                        ButtonDisable,
                        children![(
                            Text::new("Play again"),
                            TextFont {
                                font_size: 33.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        )],
                    ));

                    parent.spawn((
                        Button,
                        Node {
                            height: Val::Px(65.0),
                            margin: UiRect::all(Val::Px(20.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(NORMAL_BUTTON),
                        ScoreboardButtonAction::LeaveRoom,
                        children![(
                            Text::new("Back to menu"),
                            TextFont {
                                font_size: 33.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        )],
                    ));
                });
        });
}

#[allow(clippy::too_many_arguments)]
pub fn scoreboard_update(
    mut commands: Commands,
    final_scores: Res<FinalScores>,
    room_creator: Res<RoomCreator>,
    player_data: Res<Persistent<PlayerData>>,
    query_list: Query<Entity, With<ScoreboardList>>,
    mut query_title: Query<&mut Text, With<ScoreboardTitle>>,
    query_buttons: Query<(Entity, &ScoreboardButtonAction), With<Button>>,
    children_query: Query<&Children>,
) {
    if !final_scores.is_changed() && !room_creator.is_changed() {
        return;
    }

    let i_am_creator = room_creator.0 == Some(player_data.uuid);
    for (entity, action) in &query_buttons {
        if matches!(action, ScoreboardButtonAction::PlayAgain) {
            if i_am_creator {
                commands.entity(entity).remove::<ButtonDisable>();
            } else {
                commands.entity(entity).insert(ButtonDisable);
            }
        }
    }

    if !final_scores.is_changed() {
        return;
    }
    if let Ok(list) = query_list.single() {
        if let Ok(children) = children_query.get(list) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        let mut entries = final_scores.0.clone();
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.total));

        if let Some(winner) = entries.first() {
            for mut text in &mut query_title {
                *text = Text::new(format!("Scoreboard — winner: {}", winner.player.name));
            }
        }

        for (rank, entry) in entries.iter().enumerate() {
            let is_winner = rank == 0;
            commands.entity(list).with_children(|parent| {
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(5.0)),
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(if is_winner {
                        HOVERED_PRESSED_BUTTON
                    } else {
                        NORMAL_BUTTON
                    }),
                    children![
                        (
                            Text::new(format!("{}. {}", rank + 1, entry.player.name)),
                            TextFont {
                                font_size: 28.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ),
                        (
                            Text::new(format!("{}", entry.total)),
                            TextFont {
                                font_size: 28.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ),
                    ],
                ));
            });
        }
    }
}

type ScoreboardActionQuery<'a, 'b, 'c> = Query<
    'c,
    'b,
    (&'a Interaction, &'a ScoreboardButtonAction),
    (Changed<Interaction>, With<Button>, Without<ButtonDisable>),
>;

pub fn scoreboard_button_action(
    interaction_query: ScoreboardActionQuery,
    mut network_command: EventWriter<NetworkCommandEvent>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                ScoreboardButtonAction::PlayAgain => {
                    network_command.write(NetworkCommandEvent(
                        shared::ServerAPICommand::RestartGame(shared::RestartGameCommand),
                    ));
                }
                ScoreboardButtonAction::LeaveRoom => {
                    network_command.write(NetworkCommandEvent(
                        shared::ServerAPICommand::LeaveRoom(shared::LeaveRoomCommand),
                    ));
                }
            }
        }
    }
}
