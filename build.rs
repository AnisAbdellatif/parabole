fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let embedded = std::env::var_os("CARGO_FEATURE_EMBEDDED_MPV").is_some();
    if embedded && std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        // Homebrew installs libmpv outside the default linker search path.
        for dir in ["/opt/homebrew/lib", "/usr/local/lib"] {
            if std::path::Path::new(dir).is_dir() {
                println!("cargo:rustc-link-search=native={dir}");
            }
        }
    }
    #[cfg(feature = "slint-ui")]
    if std::env::var_os("CARGO_FEATURE_SLINT_UI").is_some() {
        println!("cargo:rerun-if-changed=ui-slint");
        println!("cargo:rerun-if-changed=assets/icons");
        println!("cargo:rerun-if-changed=assets/fonts");
        slint_build::compile("ui-slint/app.slint").expect("compile ui-slint/app.slint");
    }
}
