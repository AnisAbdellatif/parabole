//! Trakt public API: metadata fallback when IMDb fails.
pub mod client;
pub mod meta;

/// Public app id from app.trakt.tv/settings/apps; empty = Trakt disabled.
pub const CLIENT_ID: &str = "A0SZwnpBIFywCnqcsxJkra-m-sYOFlv79xzO0_IF6P4";
