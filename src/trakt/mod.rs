//! Trakt API: public metadata search, device-flow sign-in and scrobbling.
pub mod auth;
pub mod client;
pub mod meta;
pub mod scrobble;

/// Public app id from app.trakt.tv/settings/apps; empty = Trakt disabled.
pub const CLIENT_ID: &str = "A0SZwnpBIFywCnqcsxJkra-m-sYOFlv79xzO0_IF6P4";

/// Must match the redirect URI registered on the Trakt app.
pub const REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob";
