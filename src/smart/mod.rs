//! Smart tab: same-named live streams from every source grouped into one channel,
//! best variant first. Pure logic, no DB or UI.

pub mod claimed;
#[cfg(test)]
mod tests;

use crate::db::alt_name::normalize;
use crate::model::{Quality, StreamRow};
use claimed::{claimed, display_name};
use std::cmp::Ordering;
use std::collections::HashMap;

pub struct Variant {
    pub row: StreamRow,
    pub quality: Option<Quality>,
}

pub struct Channel {
    pub name: String,
    /// Best first, never empty.
    pub variants: Vec<Variant>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bucket {
    All,
    Uhd,
    FullHd,
    Hd,
    Sd,
    HighFps,
    Unknown,
}

impl Variant {
    fn measured_height(&self) -> Option<i64> {
        self.quality.as_ref().and_then(|q| q.height)
    }

    fn height(&self) -> Option<i64> {
        self.measured_height().or_else(|| claimed(&self.row.name).0)
    }

    fn fps(&self) -> Option<f64> {
        let measured = self.quality.as_ref().and_then(|q| q.fps);
        measured.or_else(|| claimed(&self.row.name).1)
    }

    fn fails(&self) -> i64 {
        self.quality.as_ref().map_or(0, |q| q.fails)
    }

    fn startup_ms(&self) -> Option<i64> {
        self.quality.as_ref().and_then(|q| q.startup_ms)
    }
}

impl Channel {
    fn best(&self) -> &Variant {
        &self.variants[0]
    }
}

impl Bucket {
    pub const ALL: [Bucket; 7] = [
        Bucket::All,
        Bucket::Uhd,
        Bucket::FullHd,
        Bucket::Hd,
        Bucket::Sd,
        Bucket::HighFps,
        Bucket::Unknown,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Bucket::All => "All channels",
            Bucket::Uhd => "4K",
            Bucket::FullHd => "Full HD",
            Bucket::Hd => "HD",
            Bucket::Sd => "SD",
            Bucket::HighFps => "50/60 fps",
            Bucket::Unknown => "Quality unknown",
        }
    }

    pub fn index(self) -> u8 {
        Bucket::ALL.iter().position(|b| *b == self).unwrap_or(0) as u8
    }

    pub fn from_index(i: u8) -> Option<Bucket> {
        Bucket::ALL.get(i as usize).copied()
    }

    /// Judged on the channel's best variant.
    pub fn contains(self, ch: &Channel) -> bool {
        let best = ch.best();
        let h = best.height();
        match self {
            Bucket::All => true,
            Bucket::Uhd => h.is_some_and(|h| h >= 2000),
            Bucket::FullHd => h.is_some_and(|h| h >= 1000),
            Bucket::Hd => h.is_some_and(|h| h >= 700),
            Bucket::Sd => h.is_some_and(|h| (1..700).contains(&h)),
            Bucket::HighFps => best.fps().is_some_and(|f| f >= 48.0),
            Bucket::Unknown => h.is_none(),
        }
    }
}

/// Descending with `None` last.
fn desc_f64(a: Option<f64>, b: Option<f64>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => b.total_cmp(&a),
        (a, b) => b.is_some().cmp(&a.is_some()),
    }
}

fn rank(a: &Variant, b: &Variant) -> Ordering {
    (a.fails() > 0)
        .cmp(&(b.fails() > 0))
        .then_with(|| b.height().cmp(&a.height()))
        .then_with(|| {
            let (ma, mb) = (a.measured_height().is_some(), b.measured_height().is_some());
            mb.cmp(&ma)
        })
        .then_with(|| desc_f64(a.fps(), b.fps()))
        .then_with(|| {
            let key = |v: &Variant| (v.startup_ms().is_none(), v.startup_ms());
            key(a).cmp(&key(b))
        })
}

/// Channels in order of first appearance; input order breaks ranking ties.
pub fn group(variants: Vec<Variant>) -> Vec<Channel> {
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut channels: Vec<Channel> = Vec::new();
    for v in variants {
        let mut key = normalize(&v.row.name);
        if key.is_empty() {
            key = format!("#{}", v.row.id);
        }
        match index.get(&key) {
            Some(&i) => channels[i].variants.push(v),
            None => {
                index.insert(key, channels.len());
                channels.push(Channel {
                    name: String::new(),
                    variants: vec![v],
                });
            }
        }
    }
    for ch in &mut channels {
        ch.variants.sort_by(rank);
        ch.name = display_name(&ch.variants[0].row.name);
    }
    channels
}

fn codec_label(codec: &str) -> String {
    match codec {
        "h264" => "H.264".into(),
        "hevc" | "h265" => "HEVC".into(),
        "mpeg2video" => "MPEG-2".into(),
        "av1" => "AV1".into(),
        other => other.to_uppercase(),
    }
}

/// "1080p50 · HEVC · 6.2 Mb/s"; `None` when no height was measured.
pub fn quality_label(q: &Quality) -> Option<String> {
    let mut out = format!("{}p", q.height?);
    if let Some(fps) = q.fps.filter(|f| *f >= 1.0) {
        out += &format!("{}", fps.round() as i64);
    }
    if let Some(codec) = &q.video_codec {
        out += &format!(" · {}", codec_label(codec));
    }
    if let Some(b) = q.bitrate {
        out += &format!(" · {:.1} Mb/s", b / 1e6);
    }
    if q.fails > 0 {
        out += " · failed last time";
    }
    Some(out)
}

pub fn badge(ch: &Channel) -> String {
    let best = ch.best();
    let mut parts = Vec::new();
    if let Some(label) = best.quality.as_ref().and_then(quality_label) {
        parts.push(label);
    } else if let Some(h) = claimed(&best.row.name).0 {
        parts.push(format!("≈{h}p (from name)"));
    }
    if ch.variants.len() > 1 {
        parts.push(format!("{} versions", ch.variants.len()));
    }
    parts.join(" · ")
}
