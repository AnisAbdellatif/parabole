use serde::{Deserialize, Serialize};

pub type ProfileId = i64;
pub type SourceId = i64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaKind {
    #[default]
    Live,
    Movie,
    Series,
}

impl MediaKind {
    pub const ALL: [MediaKind; 3] = [MediaKind::Live, MediaKind::Movie, MediaKind::Series];

    pub fn as_str(self) -> &'static str {
        match self {
            MediaKind::Live => "live",
            MediaKind::Movie => "movie",
            MediaKind::Series => "series",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.as_str() == s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceKind {
    Xtream,
    M3uUrl,
    M3uFile,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Xtream => "xtream",
            SourceKind::M3uUrl => "m3u_url",
            SourceKind::M3uFile => "m3u_file",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [SourceKind::Xtream, SourceKind::M3uUrl, SourceKind::M3uFile]
            .into_iter()
            .find(|k| k.as_str() == s)
    }
}

/// A configured playlist. For Xtream, `url` is the normalized base (`scheme://host:port`).
/// For `M3uFile`, `url` is a filesystem path.
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub id: SourceId,
    pub name: String,
    pub kind: SourceKind,
    pub url: String,
    /// Extra Xtream base URLs of the same provider, tried when `url` fails.
    pub mirrors: Vec<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub user_agent: Option<String>,
    /// Xtream live output: "ts" | "m3u8"; None = auto.
    pub output_format: Option<String>,
    pub last_refresh: Option<i64>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub id: ProfileId,
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Headers {
    pub user_agent: Option<String>,
    pub referrer: Option<String>,
    pub origin: Option<String>,
}

/// Result of classifying user-pasted text in the add-source dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceInput {
    Xtream {
        base: String,
        username: String,
        password: String,
    },
    M3uUrl(String),
    M3uFile(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedCategory {
    pub media: MediaKind,
    /// Stable provider key: Xtream `category_id`, M3U group title.
    pub ext_key: String,
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParsedStream {
    pub media: MediaKind,
    /// Stable provider key (see PLAN.md §4). Unique per (source, media).
    pub ext_key: String,
    pub category_key: Option<String>,
    pub name: String,
    pub num: Option<i64>,
    pub logo: Option<String>,
    /// M3U only. Xtream URLs are built at play time.
    pub url: Option<String>,
    pub container_ext: Option<String>,
    pub epg_id: Option<String>,
    pub archive_days: Option<i64>,
    pub headers: Headers,
    pub extra_json: Option<String>,
}

/// Everything a refresh produces. Vec order is the provider order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParsedPlaylist {
    pub categories: Vec<ParsedCategory>,
    pub streams: Vec<ParsedStream>,
    pub epg_url: Option<String>,
    pub account_json: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CategoryRow {
    pub source_id: SourceId,
    pub media: MediaKind,
    pub key: String,
    pub name: String,
    pub count: i64,
    pub hidden: bool,
    pub pinned: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StreamRow {
    pub id: i64,
    pub source_id: SourceId,
    pub media: MediaKind,
    pub key: String,
    pub name: String,
    pub num: Option<i64>,
    pub logo: Option<String>,
    pub favorite: bool,
    pub hidden: bool,
    /// A favorite whose stream the provider no longer lists; not playable.
    pub missing: bool,
}

/// What the player layer needs to resolve a stream URL.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayInfo {
    pub source_id: SourceId,
    pub media: MediaKind,
    pub key: String,
    pub name: String,
    pub url: Option<String>,
    pub container_ext: Option<String>,
    pub headers: Headers,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Season {
    pub number: i64,
    pub name: String,
    pub episodes: Vec<Episode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Episode {
    pub id: String,
    pub title: String,
    pub episode_num: Option<i64>,
    pub container_ext: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayRequest {
    pub url: String,
    pub title: String,
    pub headers: Headers,
    pub media: MediaKind,
    /// Resume offset for VOD.
    pub start_s: Option<f64>,
}

/// Details of a movie or series, all optional because providers fill them inconsistently.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MediaInfo {
    pub plot: Option<String>,
    pub cast: Option<String>,
    pub director: Option<String>,
    pub genre: Option<String>,
    pub release_date: Option<String>,
    pub duration: Option<String>,
    pub rating: Option<String>,
    pub cover: Option<String>,
    pub backdrop: Option<String>,
}
