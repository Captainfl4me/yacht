//! This plugin will display the main menu screen
use crate::colors::{
    BACKGROUND_COLOR, HOVERED_BUTTON, HOVERED_PRESSED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON,
    TEXT_COLOR,
};

use super::{despawn_screen, GameState};
use bevy::prelude::*;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    Main,
    CreateRoom,
    JoinRoom,
    #[default]
    Disabled,
}
#[derive(Component)]
enum MenuButtonAction {
    GoToCreateRoom,
    GoToRoomList,
    CreateRoom,
    JoinRoom,
    BackToMainMenu,
    Quit,
}

pub fn menu_plugin(app: &mut App) {
    app.init_state::<MenuState>()
        .add_systems(OnEnter(GameState::Menu), menu_setup)
        .add_systems(OnEnter(MenuState::Main), main_menu_setup)
        .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
        .add_systems(OnEnter(MenuState::CreateRoom), create_room_menu_setup)
        .add_systems(
            OnExit(MenuState::CreateRoom),
            despawn_screen::<OnCreateRoomMenuScreen>,
        )
        .add_systems(OnEnter(MenuState::JoinRoom), join_room_menu_setup)
        .add_systems(
            OnExit(MenuState::JoinRoom),
            despawn_screen::<OnJoinRoomMenuScreen>,
        )
        .add_systems(
            Update,
            (menu_action, button_system).run_if(in_state(GameState::Menu)),
        );
}

#[derive(Component)]
struct TopLevelMenuScreen;

#[derive(Component)]
struct SubSceneParentNode;

#[derive(Component)]
struct OnMainMenuScreen;
#[derive(Component)]
struct OnCreateRoomMenuScreen;
#[derive(Component)]
struct OnJoinRoomMenuScreen;

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
                        align_items: AlignItems::Center,
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
                            width: Val::Px(300.0),
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

fn create_room_menu_setup(mut commands: Commands, query: Query<Entity, With<SubSceneParentNode>>) {
    if let Some(sub_scene_node) = query.iter().next() {
        commands.entity(sub_scene_node).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Start,
                        ..default()
                    },
                    OnCreateRoomMenuScreen,
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

fn join_room_menu_setup(mut commands: Commands, query: Query<Entity, With<SubSceneParentNode>>) {
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
    } else {
        error!("Subscene setup occurs before Menu setup!");
    }
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.send(AppExit::Success);
                }
                MenuButtonAction::GoToCreateRoom => menu_state.set(MenuState::CreateRoom),
                MenuButtonAction::GoToRoomList => menu_state.set(MenuState::JoinRoom),
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                _ => menu_state.set(MenuState::Main),
            }
        }
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color) in &mut interaction_query {
        *background_color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}
