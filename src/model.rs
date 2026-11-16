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

/// One TV guide entry. `channel` is the XMLTV channel id, lowercased; times are unix seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Programme {
    pub channel: String,
    pub start: i64,
    pub stop: i64,
    pub title: String,
    pub descr: Option<String>,
}

pub type GuideId = i64;

/// A `<channel>` of an XMLTV guide: lowercased id and its display names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GuideChannel {
    pub id: String,
    pub names: Vec<String>,
}

/// A TV guide: the provider's own (`source_id` set, URL derived at fetch time) or a user-added URL.
#[derive(Clone, Debug, PartialEq)]
pub struct Guide {
    pub id: GuideId,
    pub source_id: Option<SourceId>,
    pub url: Option<String>,
    pub name: String,
    pub enabled: bool,
    pub position: i64,
    pub refreshed_at: Option<i64>,
    pub last_error: Option<String>,
}

/// What guide matching needs to know about one live stream.
#[derive(Clone, Debug, PartialEq)]
pub struct StreamGuideKey {
    pub source_id: SourceId,
    pub stream_key: String,
    pub epg_id: Option<String>,
    pub name: String,
}

/// A live stream (by provider key) shown with one guide channel.
#[derive(Clone, Debug, PartialEq)]
pub struct EpgLink {
    pub source_id: SourceId,
    pub stream_key: String,
    pub guide_id: GuideId,
    pub channel: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NowNext {
    pub now: Programme,
    pub next: Option<Programme>,
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
    /// Watched fraction (0..1) of a movie, or of the episode in progress for a series.
    pub progress: Option<f32>,
    /// Continue-watching series rows: the episode to pick up.
    pub resume_episode: Option<EpisodeRef>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EpisodeRef {
    pub id: String,
    pub label: String,
}

/// A user-made, ordered list of streams of one media kind.
#[derive(Clone, Debug, PartialEq)]
pub struct StreamGroup {
    pub id: i64,
    pub name: String,
    pub count: i64,
}

/// The same live channel offered by another source of the profile.
#[derive(Clone, Debug, PartialEq)]
pub struct Alternate {
    pub stream_id: i64,
    pub source_name: String,
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

/// Online details of a movie or series (IMDb, or Trakt when IMDb fails).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    /// Display name of where it came from: "IMDb" or "Trakt".
    pub origin: String,
    pub imdb_id: Option<String>,
    pub title: String,
    pub year: Option<i64>,
    /// 0..10
    pub rating: Option<f32>,
    pub votes: Option<i64>,
    pub plot: Option<String>,
    pub poster: Option<String>,
    pub genres: Vec<String>,
    pub runtime_min: Option<i64>,
    pub certificate: Option<String>,
    /// Directors for movies, creators for series.
    pub directors: Vec<String>,
    pub cast: Vec<String>,
}

/// A football match for the Football tab, independent of the provider it came from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Match {
    /// Provider-prefixed id, e.g. "s365:4672273" or "livescore:1738288".
    pub key: String,
    pub competition: String,
    pub home: String,
    /// Empty for single events (a race, a show); `home` then holds the event name.
    pub away: String,
    /// Unix seconds, UTC.
    pub kickoff_utc: i64,
    pub status: MatchStatus,
    pub broadcasts: Vec<Broadcast>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MatchStatus {
    Upcoming,
    /// `clock` is the provider's display text, e.g. "67'" or "HT"; scores are display text
    /// too ("2", tennis sets "1").
    Live {
        clock: String,
        home: String,
        away: String,
    },
    Finished,
}

/// One channel showing a match in one country.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Broadcast {
    pub channel: String,
    /// ISO 3166-1 alpha-2, uppercase.
    pub country: String,
    /// ISO 639-1, None when unknown or multilingual. Always inferred, never given by a provider.
    pub lang: Option<String>,
    /// Provider display name, e.g. "365scores".
    pub provider: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub name: String,
    pub ok: bool,
    pub error: Option<String>,
}

/// Playback quality measured by the embedded player for one live stream.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Quality {
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub fps: Option<f64>,
    pub video_codec: Option<String>,
    /// Bits per second.
    pub bitrate: Option<f64>,
    pub startup_ms: Option<i64>,
    /// Failures since the last successful play.
    pub fails: i64,
}
