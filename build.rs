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
        // Element ids are only queryable (headless tests) when debug info is emitted; release stays lean.
        let debug = std::env::var("PROFILE").as_deref() == Ok("debug");
        let config = slint_build::CompilerConfiguration::new().with_debug_info(debug);
        slint_build::compile_with_config("ui-slint/app.slint", config)
            .expect("compile ui-slint/app.slint");
    }
}
