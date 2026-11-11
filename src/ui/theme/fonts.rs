use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
use egui::{FontData, FontFamily, FontId};

const MEDIUM: &str = "inter-medium";
const SEMIBOLD: &str = "inter-semibold";

// `add_font` keeps egui's default and system fallback fonts (CJK, Arabic, emoji) behind these.
pub fn install(ctx: &egui::Context) {
    let regular = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
    let medium = include_bytes!("../../../assets/fonts/Inter-Medium.ttf");
    let semibold = include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf");
    add(
        ctx,
        "inter-regular",
        regular,
        FontPriority::Highest,
        &[FontFamily::Proportional],
    );
    add(
        ctx,
        MEDIUM,
        medium,
        FontPriority::Highest,
        &[medium_family()],
    );
    add(
        ctx,
        SEMIBOLD,
        semibold,
        FontPriority::Highest,
        &[semibold_family()],
    );
    // Playlist names use superscript letters (ᴿᴬᵂ ⁶⁰ᶠᵖˢ) that Inter lacks and egui's system-font lookup misses.
    let dejavu = include_bytes!("../../../assets/fonts/DejaVuSans.ttf");
    let all = [FontFamily::Proportional, medium_family(), semibold_family()];
    add(ctx, "dejavu-sans", dejavu, FontPriority::Lowest, &all);
}

fn add(
    ctx: &egui::Context,
    name: &str,
    bytes: &'static [u8],
    priority: FontPriority,
    families: &[FontFamily],
) {
    let families = families
        .iter()
        .map(|family| InsertFontFamily {
            family: family.clone(),
            priority,
        })
        .collect();
    ctx.add_font(FontInsert::new(
        name,
        FontData::from_static(bytes),
        families,
    ));
}

fn medium_family() -> FontFamily {
    FontFamily::Name(MEDIUM.into())
}

fn semibold_family() -> FontFamily {
    FontFamily::Name(SEMIBOLD.into())
}

pub fn regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn medium(size: f32) -> FontId {
    FontId::new(size, medium_family())
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, semibold_family())
}
