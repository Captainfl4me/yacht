use super::{MenuButtonAction, SubSceneParentNode, SubMenuScreen};
use crate::colors::{NORMAL_BUTTON, TEXT_COLOR};
use bevy::prelude::*;

pub fn create_room_menu_setup(mut commands: Commands, query: Query<Entity, With<SubSceneParentNode>>) {
    if let Some(sub_scene_node) = query.iter().next() {
        commands.entity(sub_scene_node).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Start,
                        ..default()
                    },
                    SubMenuScreen,
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
    } else {
        error!("Subscene setup occurs before Menu setup!");
    }
}
