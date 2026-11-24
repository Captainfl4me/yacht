use bevy::prelude::*;

// Balatro-inspired pixel-art theme: deep purple felt, cream cards with thick
// ink outlines and hard offset shadows, red primary buttons, gold highlights.
pub const BG_COLOR: Color = Color::srgb(0.09, 0.045, 0.19);

pub const INK: Color = Color::srgb(0.086, 0.043, 0.18);
pub const CARD: Color = Color::srgb(0.965, 0.925, 0.83);
pub const CARD_SHADOW: Color = Color::srgb(0.05, 0.02, 0.13);
pub const CARD_BORDER: Color = INK;

pub const TEXT_COLOR: Color = CARD; // light text on dark background
pub const TEXT_DIM: Color = Color::srgb(0.66, 0.58, 0.84);
pub const TEXT_INK: Color = INK; // dark text on cream cards
pub const TEXT_INK_DIM: Color = Color::srgb(0.45, 0.38, 0.62);
pub const ERROR_TEXT: Color = Color::srgb(0.78, 0.22, 0.22);

pub const BUTTON_BG: Color = Color::srgb(1.0, 0.99, 0.96);
pub const BUTTON_HOVER: Color = Color::srgb(1.0, 0.91, 0.66);
pub const BUTTON_PRESSED: Color = Color::srgb(0.85, 0.78, 0.62);
pub const PRIMARY: Color = Color::srgb(0.88, 0.27, 0.28);
pub const PRIMARY_HOVER: Color = Color::srgb(0.96, 0.38, 0.38);
pub const PRIMARY_PRESSED: Color = Color::srgb(0.68, 0.18, 0.2);
pub const PRIMARY_TEXT: Color = CARD;
pub const DISABLED_BUTTON: Color = Color::srgb(0.82, 0.77, 0.66);
pub const ACCENT: Color = Color::srgb(0.95, 0.72, 0.2); // gold

pub const NORMAL_BUTTON: Color = BUTTON_BG;
pub const HOVERED_BUTTON: Color = BUTTON_HOVER;
pub const PRESSED_BUTTON: Color = BUTTON_PRESSED;
pub const HOVERED_PRESSED_BUTTON: Color = ACCENT;
pub const PANEL_COLOR: Color = CARD;
pub const PANEL_BORDER: Color = INK;
pub const INPUT_BG: Color = BUTTON_BG;

// Kept for compatibility with existing scene code that paints panels with it.
pub const BACKGROUND_COLOR: Color = CARD;

pub const DICE_COLOR: Color = Color::srgb(1.0, 0.99, 0.96);
pub const DICE_BORDER: Color = INK;
pub const DICE_SAVED_COLOR: Color = ACCENT;
pub const PIP_COLOR: Color = INK;
