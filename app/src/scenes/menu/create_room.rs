use super::{MenuButtonAction, SubMenuScreen, SubSceneParentNode};
use crate::{
    colors::NORMAL_BUTTON,
    network::{events::CreateRoomEvent, NetworkCommandEvent},
    scenes::AppState,
    style, PrimaryButton,
};
use bevy::prelude::*;
use bevy_simple_text_input::{
    TextInput, TextInputPlaceholder, TextInputSettings, TextInputTextFont,
};
use shared::{CreateRoomCommand, CreateRoomResponse, ServerAPICommand};

#[derive(Resource)]
pub struct CreatingRoomName(pub String);

pub fn create_room_menu_setup(
    mut commands: Commands,
    query: Query<Entity, With<SubSceneParentNode>>,
) {
    if let Some(sub_scene_node) = query.iter().next() {
        commands.entity(sub_scene_node).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    SubMenuScreen,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextInput,
                        TextInputPlaceholder {
                            value: "Room name".to_string(),
                            ..default()
                        },
                        TextInputTextFont(TextFont {
                            font_size: 24.0,
                            ..default()
                        }),
                        TextInputSettings {
                            retain_on_submit: true,
                            ..default()
                        },
                        style::input_node(),
                    ));

                    let mut create_button = parent.spawn((
                        Button,
                        style::button_node(),
                        style::button_border(),
                        style::radius(),
                        BackgroundColor(NORMAL_BUTTON),
                        MenuButtonAction::CreateRoom,
                    ));
                    create_button.insert(PrimaryButton);
                    create_button.with_children(|parent| {
                        parent.spawn(style::label_on_accent("Create", 26.0));
                    });

                    parent
                        .spawn((
                            Button,
                            style::button_node(),
                            style::button_border(),
                            style::radius(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::BackToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn(style::label("Back", 26.0));
                        });
                });
        });
    } else {
        error!("Subscene setup occurs before Menu setup!");
    }
}

pub fn creating_room_setup(
    mut commands: Commands,
    query: Query<Entity, With<SubSceneParentNode>>,
    mut network_command_event: EventWriter<NetworkCommandEvent>,
    room_name: Res<CreatingRoomName>,
) {
    if let Some(sub_scene_node) = query.iter().next() {
        info!("create room: {}", room_name.0);
        network_command_event.write(NetworkCommandEvent(ServerAPICommand::CreateRoom(
            CreateRoomCommand(room_name.0.clone()),
        )));

        commands.entity(sub_scene_node).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(16.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    SubMenuScreen,
                ))
                .with_children(|parent| {
                    parent.spawn(style::label_dim("Creating…", 26.0));
                });
        });
    }
}

pub fn creating_room_update(
    mut game_state: ResMut<NextState<AppState>>,
    mut create_room_event: EventReader<CreateRoomEvent>,
) {
    if let Some(CreateRoomEvent(CreateRoomResponse(_))) = create_room_event.read().next() {
        game_state.set(AppState::Game);
    }
}
