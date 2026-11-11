//! Design tokens from the redesign mockup (dark, teal accent, Inter) and the global egui style.
#![allow(dead_code, unused_imports)] // shortcut: tokens land before their users, remove once the redesign uses them all

mod fonts;
mod icons;
mod style;

pub use fonts::{medium, regular, semibold};
pub use icons::{Icon, icon};

use egui::Color32;

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub const BG: Color32 = hex(0x101216);
pub const PANEL: Color32 = hex(0x14171c);
pub const TOOLBAR: Color32 = hex(0x171a20);
pub const CARD: Color32 = hex(0x171a20);
pub const SURFACE: Color32 = hex(0x1d2128);
pub const SURFACE_HOVER: Color32 = hex(0x262b34);
pub const ROW_HOVER: Color32 = hex(0x1a1e25);
pub const INPUT_BG: Color32 = hex(0x0c0e11);
pub const BORDER: Color32 = hex(0x2b303a);
pub const DIVIDER: Color32 = hex(0x232831);
pub const CARD_BORDER: Color32 = hex(0x262b34);
pub const CARD_BORDER_HOVER: Color32 = hex(0x3a414d);

pub const TEXT: Color32 = hex(0xe6e8ec);
pub const TEXT_ROW: Color32 = hex(0xd4d8df);
pub const TEXT_MUTED: Color32 = hex(0x9ba3b0);
pub const TEXT_DIM: Color32 = hex(0x7d8594);
pub const TEXT_FAINT: Color32 = hex(0x6b7381);
pub const BADGE_TEXT: Color32 = hex(0xb6bcc7);

pub const ACCENT: Color32 = hex(0x5eead4);
pub const ON_ACCENT: Color32 = hex(0x06201c);
pub const SELECTED_BG: Color32 = hex(0x16312f);
pub const SELECTED_BORDER: Color32 = hex(0x2f7f77);

pub const STAR: Color32 = hex(0xf5c451);
pub const LIVE_BG: Color32 = hex(0x2a1d1f);
pub const LIVE_FG: Color32 = hex(0xff9aa0);
pub const OK: Color32 = hex(0x4ade80);
pub const WARN: Color32 = hex(0xfb923c);
pub const ERROR: Color32 = hex(0xf87171);
pub const INFO_BG: Color32 = hex(0x1b2330);
pub const INFO_FG: Color32 = hex(0xa9c4ee);
pub const MATCH_BG: Color32 = hex(0x2a2616);
pub const TOAST_BG: Color32 = hex(0x232831);
pub const TOAST_BORDER: Color32 = hex(0x343b47);

pub const TOPBAR_H: f32 = 64.0;
pub const STATUSBAR_H: f32 = 30.0;
pub const SIDEBAR_W: f32 = 320.0;
pub const PLAYER_W: f32 = 420.0;
pub const CONTROL_H: f32 = 36.0;
pub const SIDEBAR_ROW_H: f32 = 38.0;
pub const CHANNEL_ROW_H: f32 = 62.0;
pub const LOGO: f32 = 44.0;
pub const POSTER_MIN_W: f32 = 150.0;
pub const GAP: f32 = 16.0;

pub const RADIUS: u8 = 8;
pub const RADIUS_L: u8 = 10;
pub const RADIUS_CARD: u8 = 12;

/// Call once at startup: fonts, colors, spacing.
pub fn install(ctx: &egui::Context) {
    fonts::install(ctx);
    style::install(ctx);
}
