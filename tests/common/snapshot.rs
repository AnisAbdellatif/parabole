#![allow(dead_code)]
//! Golden-image comparison: tolerant per channel and per pixel count, with actual/diff output on failure.
use image::{Rgba, RgbaImage};
use std::path::PathBuf;

const CHANNEL_TOLERANCE: u8 = 8;
const MAX_DIFF_RATIO: f64 = 0.003;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn differs(a: &Rgba<u8>, b: &Rgba<u8>) -> bool {
    a.0.iter().zip(b.0).any(|(x, y)| x.abs_diff(y) > CHANNEL_TOLERANCE)
}

/// Compares `img` with `tests/fixtures/slint-snapshots/<name>.png`; Err carries the failure message.
pub fn check(name: &str, img: &RgbaImage) -> Result<(), String> {
    let golden = root().join(format!("tests/fixtures/slint-snapshots/{name}.png"));
    let write_golden = || {
        std::fs::create_dir_all(golden.parent().expect("golden has a parent")).ok();
        img.save(&golden).map_err(|e| format!("{name}: write golden: {e}"))
    };
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        return write_golden();
    }
    if !golden.exists() {
        write_golden()?;
        return Err(format!("{name}: new snapshot written to {}", golden.display()));
    }
    let want = image::open(&golden)
        .map_err(|e| format!("{name}: read golden: {e}"))?
        .to_rgba8();
    let out = root().join("target/slint-snapshots");
    std::fs::create_dir_all(&out).ok();
    let actual = out.join(format!("{name}.actual.png"));
    if want.dimensions() != img.dimensions() {
        img.save(&actual).ok();
        return Err(format!(
            "{name}: size {:?} != golden {:?}; actual: {}",
            img.dimensions(),
            want.dimensions(),
            actual.display()
        ));
    }
    let mut diff = RgbaImage::new(img.width(), img.height());
    let mut bad = 0u64;
    for (x, y, p) in img.enumerate_pixels() {
        if differs(p, want.get_pixel(x, y)) {
            bad += 1;
            diff.put_pixel(x, y, Rgba([255, 0, 0, 255]));
        } else {
            diff.put_pixel(x, y, Rgba([p.0[0] / 4, p.0[1] / 4, p.0[2] / 4, 255]));
        }
    }
    let ratio = bad as f64 / (u64::from(img.width()) * u64::from(img.height())) as f64;
    if ratio <= MAX_DIFF_RATIO {
        return Ok(());
    }
    let diff_path = out.join(format!("{name}.diff.png"));
    img.save(&actual).ok();
    diff.save(&diff_path).ok();
    Err(format!(
        "{name}: {:.3}% pixels differ; actual: {}; diff: {}",
        ratio * 100.0,
        actual.display(),
        diff_path.display()
    ))
}
