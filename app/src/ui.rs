//! Shared pixel-art UI helpers: default font, pixel text/button/panel bundles.
use bevy::prelude::*;

use crate::colors::*;

/// Marker so the auto-styling system only touches each button once.
#[derive(Component)]
pub struct ButtonStyled;

/// Handle to the pixel font, loaded once at startup.
#[derive(Resource, Clone)]
pub struct PixelFont(pub Handle<Font>);

pub fn setup_default_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PixelFont(asset_server.load("fonts/PressStart2P-Regular.ttf")));
}

/// Rewrite every TextFont still using the built-in default to the pixel font.
/// Catches all screens, including text spawned later (inputs, dynamic rows).
pub fn apply_pixel_font(
    mut query: Query<&mut TextFont, Added<TextFont>>,
    font: Res<PixelFont>,
) {
    for mut text_font in &mut query {
        if text_font.font == Handle::default() {
            text_font.font = font.0.clone();
        }
    }
}

/// Give every button a chunky 4px pixel frame; hover/pressed also swap the
/// border color (background swap is handled by `button_system` in main.rs).
pub fn style_buttons(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Node), (With<Button>, Without<ButtonStyled>)>,
) {
    for (entity, mut node) in &mut query {
        node.border = UiRect::all(Val::Px(4.0));
        commands.entity(entity).insert((BorderColor(BORDER_COLOR), ButtonStyled));
    }
}

/// Border color feedback on hover/press, paired with `button_system`.
pub fn button_border_system(
    mut query: Query<(&Interaction, &mut BorderColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut border) in &mut query {
        *border = match *interaction {
            Interaction::Pressed | Interaction::Hovered => BorderColor(BORDER_HOVER_COLOR),
            Interaction::None => BorderColor(BORDER_COLOR),
        };
    }
}

/// A pixel-styled text bundle (uses the default pixel font).
pub fn pixel_text(text: impl Into<String>, size: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(TEXT_COLOR),
    )
}

/// A pixel-styled gold title text bundle.
pub fn pixel_title(text: impl Into<String>, size: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(TITLE_COLOR),
    )
}

/// A centered panel node with pixel frame (menus, popups).
pub fn panel_node(width_px: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        width: Val::Px(width_px),
        padding: UiRect::all(Val::Px(24.0)),
        border: UiRect::all(Val::Px(4.0)),
        row_gap: Val::Px(8.0),
        ..default()
    }
}
