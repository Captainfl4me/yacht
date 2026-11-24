//! This plugin will display the main menu screen
use crate::colors::INK;
use crate::network::NetworkManagerState;
use crate::style;
use crate::{PrimaryButton, PlayerData};

use super::{despawn_screen, AppState};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_simple_text_input::{
    TextInput, TextInputPlaceholder, TextInputSettings, TextInputTextColor, TextInputTextFont,
    TextInputValue,
};

mod create_room;
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
                use_starfield_background,
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
        .add_systems(Update, menu_action.run_if(in_state(AppState::Menu)))
        .add_systems(Update, name_input_update.run_if(in_state(MenuState::Main)))
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

/// Purple starfield background outside of games.
fn use_starfield_background(
    asset_server: Res<AssetServer>,
    mut bg: Query<&mut ImageNode, With<crate::BgImage>>,
) {
    for mut image in &mut bg {
        image.image = asset_server.load("background.png");
    }
}

fn menu_setup(mut commands: Commands, mut menu_state: ResMut<NextState<MenuState>>) {
    commands
        .spawn((style::screen_root(), TopLevelMenuScreen))
        .with_children(|parent| {
            parent
                .spawn(style::card_shell(460.0))
                .with_children(|shell| {
                    shell.spawn(style::card_shadow());
                    shell
                        .spawn(style::card())
                        .with_children(|parent| {
                            parent.spawn((
                                style::label_ink("YATCH", 40.0),
                                Node {
                                    align_self: AlignSelf::Center,
                                    margin: UiRect::bottom(Val::Px(4.0)),
                                    ..default()
                                },
                            ));
                            parent.spawn((
                                style::label_ink_dim("a dice game with friends", 13.0),
                                Node {
                                    align_self: AlignSelf::Center,
                                    margin: UiRect::bottom(Val::Px(20.0)),
                                    ..default()
                                },
                            ));
                            parent.spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Stretch,
                                    ..default()
                                },
                                SubSceneParentNode,
                            ));
                        });
                });
        });

    menu_state.set(MenuState::Main);
}

fn main_menu_setup(
    mut commands: Commands,
    query: Query<Entity, With<SubSceneParentNode>>,
    player_data: Res<Persistent<PlayerData>>,
) {
    if let Some(sub_scene_node) = query.iter().next() {
        commands.entity(sub_scene_node).with_children(|parent| {
            parent.spawn((
                TextInput,
                TextInputValue(player_data.name.clone()),
                TextInputPlaceholder {
                    value: "Your name".to_string(),
                    ..default()
                },
                TextInputTextFont(TextFont {
                    font: style::pixel_font(),
                    font_size: 18.0,
                    ..default()
                }),
                TextInputTextColor(TextColor(INK)),
                TextInputSettings {
                    retain_on_submit: true,
                    ..default()
                },
                style::input_node(),
                OnMainMenuScreen,
            ));

            let buttons = [
                (MenuButtonAction::GoToCreateRoom, "CREATE ROOM", true),
                (MenuButtonAction::GoToRoomList, "JOIN ROOM", false),
            ];

            for (menu_button_action, button_text, primary) in buttons {
                let mut entity = parent.spawn((
                    Button,
                    style::button_node(),
                    style::button_border(),
                    style::radius(),
                    BackgroundColor(crate::colors::NORMAL_BUTTON),
                    menu_button_action,
                    OnMainMenuScreen,
                ));
                if primary {
                    entity.insert(PrimaryButton);
                    entity.with_children(|parent| {
                        parent.spawn(style::label_on_primary(button_text, 18.0));
                    });
                } else {
                    entity.with_children(|parent| {
                        parent.spawn(style::label_ink(button_text, 18.0));
                    });
                }
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

fn name_input_update(
    query: Query<&TextInputValue, Changed<TextInputValue>>,
    mut player_data: ResMut<Persistent<PlayerData>>,
) {
    for value in &query {
        let name = value.0.trim().to_string();
        if !name.is_empty() && name != player_data.name {
            player_data.name = name;
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
            BackgroundColor(Color::srgba(0.05, 0.02, 0.12, 0.65)),
            ZIndex(2),
            OnNetworkConnecting,
        ))
        .with_children(|parent| {
            parent
                .spawn(style::card_shell(360.0))
                .with_children(|shell| {
                    shell.spawn(style::card_shadow());
                    shell
                        .spawn(style::card())
                        .with_children(|parent| {
                            parent.spawn((
                                style::label_ink("CONNECTING…", 16.0),
                                Node {
                                    align_self: AlignSelf::Center,
                                    ..default()
                                },
                            ));
                        });
                });
        });
}
