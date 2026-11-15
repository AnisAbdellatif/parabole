#![allow(dead_code)]
//! Pixel probes for alignment asserts: where does a control's ink sit vertically?
use image::RgbaImage;

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Physical-pixel box of a logical rect.
fn px(r: &Rect, scale: f32, img: &RgbaImage) -> (u32, u32, u32, u32) {
    let clamp = |v: f32, max: u32| (v.round().max(0.0) as u32).min(max);
    (
        clamp(r.x * scale, img.width()),
        clamp(r.y * scale, img.height()),
        clamp((r.x + r.w) * scale, img.width()),
        clamp((r.y + r.h) * scale, img.height()),
    )
}

fn distance(a: &image::Rgba<u8>, b: &image::Rgba<u8>) -> u32 {
    a.0.iter()
        .zip(b.0)
        .map(|(x, y)| u32::from(x.abs_diff(y)))
        .max()
        .unwrap_or(0)
}

/// First and last pixel row (physical) inside `r` whose pixels differ from the crop's top-left pixel by more than `threshold`.
pub fn ink_rows(img: &RgbaImage, r: &Rect, scale: f32, threshold: u32) -> Option<(u32, u32)> {
    let (x0, y0, x1, y1) = px(r, scale, img);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let bg = *img.get_pixel(x0, y0);
    let rows: Vec<u32> = (y0..y1)
        .filter(|&y| (x0..x1).any(|x| distance(img.get_pixel(x, y), &bg) > threshold))
        .collect();
    Some((*rows.first()?, *rows.last()?))
}

/// Vertical centre in physical pixels, doubled so half pixels stay integers.
pub fn centre2(rows: (u32, u32)) -> i64 {
    i64::from(rows.0) + i64::from(rows.1)
}
