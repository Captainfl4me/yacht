use super::{MenuButtonAction, SubSceneParentNode};
use crate::colors::{NORMAL_BUTTON, TEXT_COLOR};
use crate::network::NetworkManager;
use shared::{ServerAPICommand, ListRoomCommand};
use bevy::prelude::*;

#[derive(Component)]
pub struct OnJoinRoomMenuScreen;

pub fn join_room_menu_setup(
    mut commands: Commands,
    query: Query<Entity, With<SubSceneParentNode>>,
    mut nm: ResMut<NetworkManager>,
) {
    if let Some(sub_scene_node) = query.iter().next() {
        commands.entity(sub_scene_node).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Start,
                        ..default()
                    },
                    OnJoinRoomMenuScreen,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(300.0),
                                height: Val::Px(65.0),
                                margin: UiRect::all(Val::Px(20.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::BackToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("back"),
                                TextFont {
                                    font_size: 33.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                });
        });

        nm.send_queue.push_back(ServerAPICommand::ListRoom(ListRoomCommand));
    } else {
        error!("Subscene setup occurs before Menu setup!");
    }
}
