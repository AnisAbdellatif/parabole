//! Pure pixel work for the image loader: backdrop blur/darken.
use image::RgbaImage;

const BLUR_SIGMA: f32 = 14.0;
const DARKEN: f32 = 0.65;

/// Blurs and darkens by 35%, for use behind text.
pub fn backdrop(img: &RgbaImage) -> RgbaImage {
    let mut out = image::imageops::blur(img, BLUR_SIGMA);
    for p in out.pixels_mut() {
        for c in &mut p.0[..3] {
            *c = (f32::from(*c) * DARKEN) as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn two_colour() -> RgbaImage {
        RgbaImage::from_fn(10, 10, |x, _| {
            if x < 5 {
                Rgba([200, 20, 20, 255])
            } else {
                Rgba([20, 20, 200, 255])
            }
        })
    }

    #[test]
    fn backdrop_darkens_and_keeps_size() {
        let img = RgbaImage::from_pixel(16, 9, Rgba([200, 100, 50, 255]));
        let out = backdrop(&img);
        assert_eq!(out.dimensions(), (16, 9));
        let p = out.get_pixel(8, 4).0;
        assert!(p[0] <= 131 && p[0] >= 125, "{p:?}");
        assert_eq!(p[3], 255);
    }

    #[test]
    fn backdrop_blurs_an_edge() {
        let img = two_colour();
        let out = backdrop(&img);
        let (l, m) = (out.get_pixel(0, 5).0, out.get_pixel(5, 5).0);
        assert!(m[0] < l[0] && m[2] > out.get_pixel(0, 5).0[2]);
    }
}
