#![cfg(feature = "slint-ui")]
#[path = "common/slint_seed.rs"]
mod slint_seed;

#[test]
fn seed_builds() {
    let dir = slint_seed::scratch_dir("seed");
    slint_seed::isolate_home(&dir);
    let path = slint_seed::seed(&dir);
    assert!(path.exists());
    slint_seed::cleanup(&dir);
}
