use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
use egui::{FontData, FontFamily, FontId};

const MEDIUM: &str = "inter-medium";
const SEMIBOLD: &str = "inter-semibold";

pub fn install(ctx: &egui::Context) {
    add(
        ctx,
        "inter-regular",
        include_bytes!("../../../assets/fonts/Inter-Regular.ttf"),
        FontFamily::Proportional,
    );
    add(
        ctx,
        MEDIUM,
        include_bytes!("../../../assets/fonts/Inter-Medium.ttf"),
        FontFamily::Name(MEDIUM.into()),
    );
    add(
        ctx,
        SEMIBOLD,
        include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf"),
        FontFamily::Name(SEMIBOLD.into()),
    );
}

// `add_font` keeps the default and system fallback fonts (CJK, Arabic, emoji) behind Inter.
fn add(ctx: &egui::Context, name: &str, bytes: &'static [u8], family: FontFamily) {
    let mut families = vec![InsertFontFamily {
        family: family.clone(),
        priority: FontPriority::Highest,
    }];
    if family != FontFamily::Proportional {
        families.push(InsertFontFamily {
            family: FontFamily::Proportional,
            priority: FontPriority::Lowest,
        });
    }
    ctx.add_font(FontInsert::new(
        name,
        FontData::from_static(bytes),
        families,
    ));
}

pub fn regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(MEDIUM.into()))
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}
