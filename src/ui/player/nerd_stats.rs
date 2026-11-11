use crate::player::Stats;
use crate::ui::theme;
use egui::{Color32, Painter, Pos2, Rect, vec2};

const PAD: f32 = 10.0;
const LINE_H: f32 = 16.0;
const COL_GAP: f32 = 12.0;
/// Leaves room for the pane's LIVE/VOD tag above.
const TOP: f32 = 40.0;

/// Translucent table in the video's top-left corner, below the media tag.
pub fn paint(painter: &Painter, video: Rect, stats: &Stats) {
    let lines = stats.lines();
    let font = theme::regular(11.5);
    let galleys: Vec<_> = lines
        .iter()
        .map(|(label, value)| {
            let l = painter.layout_no_wrap(label.to_string(), font.clone(), theme::TEXT_MUTED);
            let v = painter.layout_no_wrap(value.clone(), font.clone(), theme::TEXT);
            (l, v)
        })
        .collect();
    let label_w = galleys.iter().map(|(l, _)| l.size().x).fold(0.0, f32::max);
    let value_w = galleys.iter().map(|(_, v)| v.size().x).fold(0.0, f32::max);
    let size =
        vec2(label_w + COL_GAP + value_w, LINE_H * galleys.len() as f32) + vec2(PAD, PAD) * 2.0;
    let rect = Rect::from_min_size(video.min + vec2(12.0, TOP), size).intersect(video);
    let painter = painter.with_clip_rect(rect);
    painter.rect_filled(rect, theme::RADIUS, Color32::from_black_alpha(180));
    for (i, (l, v)) in galleys.into_iter().enumerate() {
        let y = rect.top() + PAD + LINE_H * i as f32;
        painter.galley(Pos2::new(rect.left() + PAD, y), l, theme::TEXT_MUTED);
        painter.galley(
            Pos2::new(rect.left() + PAD + label_w + COL_GAP, y),
            v,
            theme::TEXT,
        );
    }
}
