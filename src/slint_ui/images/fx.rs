//! Pure pixel work for the image loader: backdrop blur/darken and dominant colour.
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

/// Saturation-weighted mean colour, skipping near-black and near-white pixels.
pub fn dominant(img: &RgbaImage) -> Option<[u8; 3]> {
    let (mut sum, mut total) = ([0f64; 3], 0f64);
    for p in img.pixels() {
        let [r, g, b, a] = p.0;
        let (max, min) = (r.max(g).max(b), r.min(g).min(b));
        if a < 128 || max < 30 || min > 225 || max == min {
            continue;
        }
        let w = f64::from(max - min) / f64::from(max);
        for (s, c) in sum.iter_mut().zip([r, g, b]) {
            *s += w * f64::from(c);
        }
        total += w;
    }
    (total > 0.0).then(|| sum.map(|s| (s / total).round() as u8))
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
    fn dominant_is_mean_of_equally_saturated_colours() {
        let [r, g, b] = dominant(&two_colour()).unwrap();
        assert_eq!((r, g, b), (110, 20, 110));
    }

    #[test]
    fn dominant_skips_black_white_and_grey() {
        let mut img = RgbaImage::from_pixel(8, 8, Rgba([0, 0, 0, 255]));
        img.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
        img.put_pixel(1, 0, Rgba([128, 128, 128, 255]));
        assert_eq!(dominant(&img), None);
        img.put_pixel(2, 0, Rgba([10, 200, 10, 255]));
        assert_eq!(dominant(&img), Some([10, 200, 10]));
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
