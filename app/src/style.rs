use bevy::prelude::*;

use crate::colors::*;

pub fn radius() -> BorderRadius {
    BorderRadius::all(Val::Px(16.0))
}

pub fn radius_sm() -> BorderRadius {
    BorderRadius::all(Val::Px(10.0))
}

/// Full-screen root node that centers its child.
pub fn screen_root() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

/// Rounded, bordered card panel of the given width.
pub fn card(width: f32) -> (Node, BackgroundColor, BorderColor, BorderRadius) {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            width: Val::Px(width),
            padding: UiRect::all(Val::Px(28.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(PANEL_COLOR),
        BorderColor(PANEL_BORDER),
        radius(),
    )
}

pub fn button_node() -> Node {
    Node {
        height: Val::Px(56.0),
        margin: UiRect::vertical(Val::Px(8.0)),
        padding: UiRect::horizontal(Val::Px(24.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

pub fn button_border() -> BorderColor {
    BorderColor(PANEL_BORDER)
}

pub fn input_node() -> (Node, BackgroundColor, BorderColor, BorderRadius) {
    (
        Node {
            height: Val::Px(56.0),
            margin: UiRect::vertical(Val::Px(8.0)),
            padding: UiRect::horizontal(Val::Px(16.0)),
            border: UiRect::all(Val::Px(2.0)),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(INPUT_BG),
        BorderColor(PANEL_BORDER),
        radius_sm(),
    )
}

pub fn label(text: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    (
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(TEXT_COLOR),
    )
}

pub fn label_dim(text: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    (
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(TEXT_DIM),
    )
}

pub fn label_on_accent(text: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    (
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(ACCENT_TEXT),
    )
}

pub fn label_accent(text: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    (
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(ACCENT),
    )
}

/// bevy_simple_text_input positions its placeholder absolutely at the top-left
/// of the input; shift it to the vertical center so it matches the typed text.
pub fn center_text_input_placeholder(
    inputs: Query<&Children, With<bevy_simple_text_input::TextInput>>,
    mut nodes: Query<&mut Node>,
) {
    for children in &inputs {
        for child in children.iter() {
            if let Ok(mut node) = nodes.get_mut(child) {
                if node.position_type == PositionType::Absolute {
                    node.top = Val::Percent(50.0);
                    node.margin.top = Val::Px(-14.0);
                }
            }
        }
    }
}
