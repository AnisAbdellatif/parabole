use egui::{Color32, Image, ImageSource, include_image};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Search,
    Settings,
    Star,
    StarFilled,
    Recent,
    Eye,
    EyeOff,
    Grip,
    Play,
    Pause,
    Stop,
    Volume,
    VolumeOff,
    Fullscreen,
    FullscreenExit,
    Close,
    DockRight,
    DockBottom,
    Live,
    Back,
    Info,
    Subtitles,
    Audio,
    Mini,
}

fn source(i: Icon) -> ImageSource<'static> {
    match i {
        Icon::Search => include_image!("../../../assets/icons/search.svg"),
        Icon::Settings => include_image!("../../../assets/icons/settings.svg"),
        Icon::Star => include_image!("../../../assets/icons/star.svg"),
        Icon::StarFilled => include_image!("../../../assets/icons/star_filled.svg"),
        Icon::Recent => include_image!("../../../assets/icons/recent.svg"),
        Icon::Eye => include_image!("../../../assets/icons/eye.svg"),
        Icon::EyeOff => include_image!("../../../assets/icons/eye_off.svg"),
        Icon::Grip => include_image!("../../../assets/icons/grip.svg"),
        Icon::Play => include_image!("../../../assets/icons/play.svg"),
        Icon::Pause => include_image!("../../../assets/icons/pause.svg"),
        Icon::Stop => include_image!("../../../assets/icons/stop.svg"),
        Icon::Volume => include_image!("../../../assets/icons/volume.svg"),
        Icon::VolumeOff => include_image!("../../../assets/icons/volume_off.svg"),
        Icon::Fullscreen => include_image!("../../../assets/icons/fullscreen.svg"),
        Icon::FullscreenExit => include_image!("../../../assets/icons/fullscreen_exit.svg"),
        Icon::Close => include_image!("../../../assets/icons/close.svg"),
        Icon::DockRight => include_image!("../../../assets/icons/dock_right.svg"),
        Icon::DockBottom => include_image!("../../../assets/icons/dock_bottom.svg"),
        Icon::Live => include_image!("../../../assets/icons/live.svg"),
        Icon::Back => include_image!("../../../assets/icons/back.svg"),
        Icon::Info => include_image!("../../../assets/icons/info.svg"),
        Icon::Subtitles => include_image!("../../../assets/icons/subtitles.svg"),
        Icon::Audio => include_image!("../../../assets/icons/audio.svg"),
        Icon::Mini => include_image!("../../../assets/icons/mini.svg"),
    }
}

/// SVGs are drawn in white, so `tint` sets the final color.
pub fn icon(i: Icon, size: f32, tint: Color32) -> Image<'static> {
    Image::new(source(i))
        .fit_to_exact_size(egui::vec2(size, size))
        .tint(tint)
}
