use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::{
    colors::{BUTTON_BG, HOVERED_PRESSED_BUTTON, INK, NORMAL_BUTTON},
    network::NetworkCommandEvent,
    style, ButtonDisable, PlayerData, PrimaryButton,
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
        .spawn((style::screen_root(), TopLevelScoreboardScreen))
        .with_children(|parent| {
            parent
                .spawn(style::card_shell(540.0))
                .with_children(|shell| {
                    shell.spawn(style::card_shadow());
                    shell
                        .spawn(style::card())
                        .with_children(|parent| {
                    parent.spawn((
                        style::label_ink("SCOREBOARD", 28.0),
                        Node {
                            align_self: AlignSelf::Center,
                            margin: UiRect::bottom(Val::Px(20.0)),
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

                    let mut play_again = parent.spawn((
                        Button,
                        style::button_node(),
                        style::radius(),
                        BackgroundColor(NORMAL_BUTTON),
                        ScoreboardButtonAction::PlayAgain,
                        ButtonDisable,
                    ));
                    play_again.insert(PrimaryButton);
                    play_again.with_children(|parent| {
                        parent.spawn(style::label_on_primary("PLAY AGAIN", 18.0));
                    });

                    parent
                        .spawn((
                            Button,
                            style::button_node(),
                            style::button_border(),
                            style::radius(),
                            BackgroundColor(NORMAL_BUTTON),
                            ScoreboardButtonAction::LeaveRoom,
                        ))
                        .with_children(|parent| {
                            parent.spawn(style::label_ink("BACK TO MENU", 18.0));
                        });
                    });
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
                *text = Text::new(format!("WINNER: {}", winner.player.name));
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
                        padding: UiRect::all(Val::Px(10.0)),
                        margin: UiRect::vertical(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(if is_winner {
                        HOVERED_PRESSED_BUTTON
                    } else {
                        BUTTON_BG
                    }),
                    BorderColor(INK),
                    style::radius_sm(),
                    children![
                        style::label_ink(format!("{}. {}", rank + 1, entry.player.name), 16.0),
                        style::label_ink(format!("{}", entry.total), 16.0),
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
