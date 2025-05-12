//! This plugin will display the main menu screen
use crate::colors::{BACKGROUND_COLOR, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, TEXT_COLOR};
use crate::network::NetworkManagerState;

use super::{despawn_screen, AppState};
use bevy::prelude::*;

mod create_room;
use bevy_simple_text_input::TextInputValue;
use create_room::{
    create_room_menu_setup, creating_room_setup, creating_room_update, CreatingRoomName,
};
mod join_room;
use join_room::{
    join_room_menu_setup, join_room_update, joining_room_setup, joining_room_update,
    JoiningRoomUuid,
};

#[derive(Component)]
pub struct SubMenuScreen;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    Main,
    CreateRoom,
    CreatingRoom,
    JoinRoom,
    JoiningRoom,
    #[default]
    Disabled,
}
#[derive(Component)]
enum MenuButtonAction {
    GoToCreateRoom,
    GoToRoomList,
    CreateRoom,
    JoinRoom(uuid::Uuid),
    BackToMainMenu,
    Quit,
}

pub fn menu_plugin(app: &mut App) {
    app.init_state::<MenuState>()
        .add_systems(
            OnEnter(AppState::Menu),
            (
                menu_setup,
                network_connecting_pop_up.run_if(not(in_state(NetworkManagerState::Connected))),
            ),
        )
        .add_systems(OnEnter(MenuState::Main), main_menu_setup)
        .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
        .add_systems(OnEnter(MenuState::CreateRoom), create_room_menu_setup)
        .add_systems(OnEnter(MenuState::JoinRoom), join_room_menu_setup)
        .add_systems(OnEnter(MenuState::JoiningRoom), joining_room_setup)
        .add_systems(OnEnter(MenuState::CreatingRoom), creating_room_setup)
        .add_systems(
            Update,
            join_room_update.run_if(in_state(MenuState::JoinRoom)),
        )
        .add_systems(
            Update,
            joining_room_update.run_if(in_state(MenuState::JoiningRoom)),
        )
        .add_systems(
            Update,
            creating_room_update.run_if(in_state(MenuState::CreatingRoom)),
        )
        .add_systems(OnExit(MenuState::JoinRoom), despawn_screen::<SubMenuScreen>)
        .add_systems(
            OnExit(MenuState::CreateRoom),
            despawn_screen::<SubMenuScreen>,
        )
        .add_systems(
            OnExit(MenuState::JoiningRoom),
            despawn_screen::<SubMenuScreen>,
        )
        .add_systems(
            OnExit(MenuState::CreatingRoom),
            despawn_screen::<SubMenuScreen>,
        )
        .add_systems(
            Update,
            (menu_action, button_system).run_if(in_state(AppState::Menu)),
        )
        .add_systems(
            OnEnter(NetworkManagerState::Disconnect),
            network_connecting_pop_up,
        )
        .add_systems(
            OnEnter(NetworkManagerState::Connected),
            despawn_screen::<OnNetworkConnecting>,
        )
        .add_systems(OnExit(AppState::Menu), despawn_screen::<TopLevelMenuScreen>);
}

#[derive(Component)]
struct TopLevelMenuScreen;

#[derive(Component)]
struct SubSceneParentNode;

#[derive(Component)]
struct OnMainMenuScreen;

#[derive(Component)]
struct OnNetworkConnecting;

fn menu_setup(mut commands: Commands, mut menu_state: ResMut<NextState<MenuState>>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Center,
                ..default()
            },
            TopLevelMenuScreen,
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
                    SubSceneParentNode,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("YATCH"),
                        TextFont {
                            font_size: 67.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                    ));
                });
        });

    menu_state.set(MenuState::Main);
}

fn main_menu_setup(mut commands: Commands, query: Query<Entity, With<SubSceneParentNode>>) {
    if let Some(sub_scene_node) = query.iter().next() {
        commands.entity(sub_scene_node).with_children(|parent| {
            let buttons = [
                (MenuButtonAction::GoToCreateRoom, "Create room"),
                (MenuButtonAction::GoToRoomList, "Join room"),
            ];

            for (menu_button_action, button_text) in buttons {
                parent
                    .spawn((
                        Button,
                        Node {
                            height: Val::Px(65.0),
                            margin: UiRect::all(Val::Px(20.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(NORMAL_BUTTON),
                        menu_button_action,
                        OnMainMenuScreen,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(button_text),
                            TextFont {
                                font_size: 33.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                    });
            }
        });
    } else {
        error!("Subscene setup occurs before Menu setup!");
    }
}

type MenuActionInteractionQueryType<'a, 'b, 'c> =
    Query<'c, 'b, (&'a Interaction, &'a MenuButtonAction), (Changed<Interaction>, With<Button>)>;

fn menu_action(
    mut commands: Commands,
    interaction_query: MenuActionInteractionQueryType,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    text_input_query: Query<&TextInputValue>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.write(AppExit::Success);
                }
                MenuButtonAction::GoToCreateRoom => menu_state.set(MenuState::CreateRoom),
                MenuButtonAction::GoToRoomList => menu_state.set(MenuState::JoinRoom),
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                MenuButtonAction::JoinRoom(uuid) => {
                    commands.insert_resource(JoiningRoomUuid(*uuid));
                    menu_state.set(MenuState::JoiningRoom)
                }
                MenuButtonAction::CreateRoom => {
                    if let Ok(room_name) = text_input_query.single() {
                        if !room_name.0.is_empty() {
                            commands.insert_resource(CreatingRoomName(room_name.0.clone()));
                            menu_state.set(MenuState::CreatingRoom);
                        }
                    }
                }
            }
        }
    }
}

type ButtonSystemInteractionQuery<'a, 'b, 'c> =
    Query<'b, 'c, (&'a Interaction, &'a mut BackgroundColor), (Changed<Interaction>, With<Button>)>;

fn button_system(mut interaction_query: ButtonSystemInteractionQuery) {
    for (interaction, mut background_color) in &mut interaction_query {
        *background_color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

fn network_connecting_pop_up(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ZIndex(2),
            OnNetworkConnecting,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BACKGROUND_COLOR),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Connecting to server..."),
                        TextFont {
                            font_size: 26.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                    ));
                });
        });
}
