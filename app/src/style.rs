use bevy::prelude::*;
use std::sync::OnceLock;

use crate::colors::*;

static PIXEL_FONT: OnceLock<Handle<Font>> = OnceLock::new();

pub fn set_pixel_font(handle: Handle<Font>) {
    let _ = PIXEL_FONT.set(handle);
}

pub fn pixel_font() -> Handle<Font> {
    PIXEL_FONT.get().cloned().unwrap_or_default()
}

pub fn radius_card() -> BorderRadius {
    BorderRadius::all(Val::Px(12.0))
}

pub fn radius() -> BorderRadius {
    BorderRadius::all(Val::Px(10.0))
}

pub fn radius_sm() -> BorderRadius {
    BorderRadius::all(Val::Px(8.0))
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

/// Marker for the outer wrapper of a card (holds the shadow + card as children).
#[derive(Component)]
pub struct CardShell;

/// Relative wrapper sized to the card; spawn `card_shadow()` then `card()` inside.
pub fn card_shell(width: f32) -> (Node, CardShell) {
    (
        Node {
            position_type: PositionType::Relative,
            width: Val::Px(width),
            ..default()
        },
        CardShell,
    )
}

/// Hard offset shadow drawn behind the card (Balatro-style drop shadow).
pub fn card_shadow() -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(7.0),
            top: Val::Px(7.0),
            right: Val::Px(-7.0),
            bottom: Val::Px(-7.0),
            ..default()
        },
        BackgroundColor(CARD_SHADOW),
        radius_card(),
    )
}

/// Cream card panel with a thick ink outline.
pub fn card() -> (Node, BackgroundColor, BorderColor, BorderRadius) {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(24.0)),
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(CARD),
        BorderColor(CARD_BORDER),
        radius_card(),
    )
}

pub fn button_node() -> Node {
    Node {
        height: Val::Px(52.0),
        margin: UiRect::vertical(Val::Px(6.0)),
        padding: UiRect::horizontal(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(3.0)),
        ..default()
    }
}

pub fn button_border() -> BorderColor {
    BorderColor(INK)
}

pub fn input_node() -> (Node, BackgroundColor, BorderColor, BorderRadius) {
    (
        Node {
            height: Val::Px(52.0),
            margin: UiRect::vertical(Val::Px(6.0)),
            padding: UiRect::horizontal(Val::Px(14.0)),
            border: UiRect::all(Val::Px(3.0)),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(INPUT_BG),
        BorderColor(INK),
        radius_sm(),
    )
}

fn text_bundle(text: impl Into<String>, size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (
        Text::new(text),
        TextFont {
            font: pixel_font(),
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}

/// Light text (on dark background).
pub fn label(text: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    text_bundle(text, size, TEXT_COLOR)
}

pub fn label_dim(text: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    text_bundle(text, size, TEXT_DIM)
}

/// Dark ink text (on cream cards/buttons).
pub fn label_ink(text: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    text_bundle(text, size, TEXT_INK)
}

pub fn label_ink_dim(text: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    text_bundle(text, size, TEXT_INK_DIM)
}

/// Cream text on red primary buttons.
pub fn label_on_primary(text: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    text_bundle(text, size, PRIMARY_TEXT)
}

/// Gold accent text (on dark background).
pub fn label_accent(text: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    text_bundle(text, size, ACCENT)
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
                    node.margin.top = Val::Px(-11.0);
                }
            }
        }
    }
}
