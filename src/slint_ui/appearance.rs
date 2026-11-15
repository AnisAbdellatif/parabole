//! Runtime theme choices: persisted in `Persisted`, pushed into the Slint `Theme` global.
use super::{AppWindow, Theme};
use serde::{Deserialize, Serialize};
use slint::{Color, ComponentHandle};

/// (accent, accent-2) pairs: teal/indigo, indigo/pink, pink/orange, orange/yellow, lime/teal, slate/sky.
pub const ACCENTS: [(u32, u32); 6] = [
    (0x5eead4, 0x818cf8),
    (0x818cf8, 0xf472b6),
    (0xf472b6, 0xfb923c),
    (0xfb923c, 0xfacc15),
    (0xa3e635, 0x5eead4),
    (0x94a3b8, 0x38bdf8),
];

const DENSITIES: [f32; 3] = [0.85, 1.0, 1.6];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Appearance {
    pub accent: u8,
    pub mode: u8,
    pub density: u8,
    pub radius_scale: f32,
    pub font_scale: f32,
    pub reduce_motion: bool,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            accent: 0,
            mode: 0,
            density: 1,
            radius_scale: 1.0,
            font_scale: 1.0,
            reduce_motion: false,
        }
    }
}

impl Appearance {
    /// Copy with every field forced into its valid range (NaN falls back to 1.0).
    pub fn clamped(&self) -> Self {
        let scale = |v: f32, lo: f32, hi: f32| if v.is_nan() { 1.0 } else { v.clamp(lo, hi) };
        Self {
            accent: self.accent.min(ACCENTS.len() as u8 - 1),
            mode: self.mode.min(2),
            density: self.density.min(DENSITIES.len() as u8 - 1),
            radius_scale: scale(self.radius_scale, 0.5, 1.5),
            font_scale: scale(self.font_scale, 0.85, 1.3),
            reduce_motion: self.reduce_motion,
        }
    }

    pub fn apply(&self, win: &AppWindow) {
        let a = self.clamped();
        let (c1, c2) = ACCENTS[a.accent as usize];
        let t = win.global::<Theme>();
        #[cfg(target_os = "macos")]
        t.set_font("Avenir Next".into());
        t.set_accent(rgb(c1));
        t.set_accent_2(rgb(c2));
        t.set_mode(a.mode as i32);
        t.set_density(DENSITIES[a.density as usize]);
        t.set_radius_scale(a.radius_scale);
        t.set_font_scale(a.font_scale);
        t.set_reduce_motion(a.reduce_motion);
    }
}

fn rgb(v: u32) -> Color {
    Color::from_rgb_u8((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_out_of_range() {
        let a = Appearance {
            accent: 99,
            mode: 7,
            density: 9,
            radius_scale: 5.0,
            font_scale: f32::NAN,
            reduce_motion: true,
        }
        .clamped();
        assert_eq!((a.accent, a.mode, a.density), (5, 2, 2));
        assert_eq!((a.radius_scale, a.font_scale), (1.5, 1.0));
    }

    #[test]
    fn serde_round_trip_and_defaults() {
        let a = Appearance {
            accent: 3,
            mode: 2,
            ..Default::default()
        };
        let back: Appearance = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        assert_eq!(a, back);
        assert_eq!(
            serde_json::from_str::<Appearance>("{}").unwrap(),
            Appearance::default()
        );
    }
}
