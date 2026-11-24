use bevy::prelude::*;

// Midnight-felt theme: deep navy background, slate cards, amber accent.
pub const BG_COLOR: Color = Color::srgb(0.043, 0.055, 0.086);
pub const PANEL_COLOR: Color = Color::srgb(0.102, 0.125, 0.18);
pub const PANEL_BORDER: Color = Color::srgb(0.235, 0.29, 0.38);
pub const INPUT_BG: Color = Color::srgb(0.066, 0.082, 0.125);

pub const ACCENT: Color = Color::srgb(0.98, 0.74, 0.25);
pub const ACCENT_HOVER: Color = Color::srgb(1.0, 0.81, 0.38);
pub const ACCENT_PRESSED: Color = Color::srgb(0.85, 0.62, 0.16);
pub const ACCENT_TEXT: Color = Color::srgb(0.12, 0.09, 0.03);

pub const TEXT_COLOR: Color = Color::srgb(0.92, 0.94, 0.97);
pub const TEXT_DIM: Color = Color::srgb(0.62, 0.68, 0.77);
pub const ERROR_TEXT: Color = Color::srgb(0.98, 0.45, 0.45);

pub const NORMAL_BUTTON: Color = Color::srgb(0.17, 0.21, 0.29);
pub const HOVERED_BUTTON: Color = Color::srgb(0.23, 0.28, 0.39);
pub const PRESSED_BUTTON: Color = ACCENT;
pub const DISABLED_BUTTON: Color = Color::srgb(0.12, 0.14, 0.19);
pub const HOVERED_PRESSED_BUTTON: Color = Color::srgba(0.98, 0.74, 0.25, 0.3);

// Kept for compatibility with existing scene code that paints panels with it.
pub const BACKGROUND_COLOR: Color = PANEL_COLOR;

pub const DICE_COLOR: Color = Color::srgb(0.96, 0.96, 0.93);
pub const DICE_BORDER: Color = Color::srgb(0.72, 0.75, 0.82);
pub const DICE_SAVED_COLOR: Color = ACCENT;
pub const PIP_COLOR: Color = Color::srgb(0.09, 0.11, 0.16);
