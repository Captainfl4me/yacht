use bevy::prelude::*;

/// Warm off-white for regular text
pub const TEXT_COLOR: Color = Color::srgb(0.96, 0.94, 0.88);
/// Gold accent for titles and pressed buttons
pub const TITLE_COLOR: Color = Color::srgb(0.95, 0.71, 0.20);
/// Dark blue-charcoal, matches pixel_bg.png tile
pub const BACKGROUND_COLOR: Color = Color::srgb(0.078, 0.102, 0.149);
/// Slightly lighter panel behind menus
pub const PANEL_COLOR: Color = Color::srgb(0.118, 0.153, 0.208);

pub const NORMAL_BUTTON: Color = Color::srgb(0.165, 0.208, 0.278);
pub const HOVERED_BUTTON: Color = Color::srgb(0.239, 0.310, 0.420);
pub const PRESSED_BUTTON: Color = Color::srgb(0.95, 0.71, 0.20);
pub const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.85, 0.60, 0.15);

/// Pixel-frame borders around buttons/panels/inputs
pub const BORDER_COLOR: Color = Color::srgb(0.353, 0.420, 0.549);
pub const BORDER_HOVER_COLOR: Color = Color::srgb(0.95, 0.71, 0.20);

pub const DICE_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
pub const DICE_SAVED_COLOR: Color = Color::srgb(0.95, 0.8, 0.3);
pub const PIP_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
