//! Registers DejaVu Sans as a last-resort fallback: Slint's own fallback ignores fonts imported in `.slint`.
use slint::fontique_011::{fontique, shared_collection};

const DEJAVU: &[u8] = include_bytes!("../../assets/fonts/DejaVuSans.ttf");
// Latin covers modifier letters like ᵁᴴᴰ; Common/Inherited cover symbols.
const SCRIPTS: [[u8; 4]; 3] = [*b"Latn", *b"Zyyy", *b"Zinh"];

pub fn install_fallback() {
    let mut collection = shared_collection();
    let fonts = collection.register_fonts(
        fontique::Blob::new(std::sync::Arc::new(DEJAVU.to_vec())),
        None,
    );
    for s in SCRIPTS {
        let key = fontique::FallbackKey::new(fontique::Script::from_bytes(s), None);
        collection.append_fallbacks(key, fonts.iter().map(|f| f.0));
    }
}
