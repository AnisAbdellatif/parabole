//! Device-flow sign-in (no client secret), token refresh and revoke.
use super::client::{get_authed, post_json};
use super::{CLIENT_ID, REDIRECT_URI};
use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub username: Option<String>,
}

/// Never prints the tokens.
impl std::fmt::Debug for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tokens")
            .field("expires_at", &self.expires_at)
            .field("username", &self.username)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub expires_in: i64,
    pub interval: i64,
}

#[derive(Debug, PartialEq)]
pub enum Poll {
    Pending,
    SlowDown,
    Done(Tokens),
    Expired,
    Denied,
    Invalid,
}

/// Reads a token response; `expires_at` = `created_at` + `expires_in`.
pub fn parse_tokens(v: &Value) -> Option<Tokens> {
    let text = |k: &str| {
        v.get(k)?
            .as_str()
            .filter(|s| !s.is_empty())
            .map(String::from)
    };
    let num = |k: &str| v.get(k)?.as_i64();
    Some(Tokens {
        access_token: text("access_token")?,
        refresh_token: text("refresh_token")?,
        expires_at: num("created_at")? + num("expires_in")?,
        username: None,
    })
}

pub fn device_code(client: &Client) -> Result<DeviceCode> {
    let (status, v) = post_json(
        client,
        "/oauth/device/code",
        &json!({ "client_id": CLIENT_ID }),
        None,
    )?;
    if status != 200 {
        bail!("Trakt refused the sign-in request (HTTP {status})");
    }
    let text = |k: &str| v.get(k)?.as_str().map(String::from);
    let code = DeviceCode {
        device_code: text("device_code").context("Trakt device code")?,
        user_code: text("user_code").context("Trakt user code")?,
        verification_url: text("verification_url").context("Trakt verification url")?,
        expires_in: v.get("expires_in").and_then(Value::as_i64).unwrap_or(600),
        interval: v.get("interval").and_then(Value::as_i64).unwrap_or(5),
    };
    if !code.verification_url.starts_with("https://") {
        bail!("Trakt sent an unexpected verification address");
    }
    Ok(code)
}

pub fn poll(client: &Client, device_code: &str) -> Result<Poll> {
    let (status, v) = post_json(
        client,
        "/oauth/device/token",
        &json!({ "code": device_code, "client_id": CLIENT_ID }),
        None,
    )?;
    Ok(match status {
        200 => Poll::Done(parse_tokens(&v).context("Trakt token response")?),
        400 => Poll::Pending,
        429 => Poll::SlowDown,
        410 => Poll::Expired,
        418 => Poll::Denied,
        404 | 409 => Poll::Invalid,
        s => bail!("Trakt sign-in failed (HTTP {s})"),
    })
}

/// `Ok(None)`: the refresh token was rejected, the user must connect again.
pub fn refresh(client: &Client, refresh_token: &str) -> Result<Option<Tokens>> {
    let (status, v) = post_json(
        client,
        "/oauth/token",
        &json!({
            "refresh_token": refresh_token,
            "client_id": CLIENT_ID,
            "redirect_uri": REDIRECT_URI,
            "grant_type": "refresh_token",
        }),
        None,
    )?;
    match status {
        200 => Ok(Some(parse_tokens(&v).context("Trakt token response")?)),
        400 | 401 => Ok(None),
        s => bail!("Trakt token refresh failed (HTTP {s})"),
    }
}

pub fn revoke(client: &Client, access_token: &str) -> Result<()> {
    post_json(
        client,
        "/oauth/revoke",
        &json!({ "token": access_token, "client_id": CLIENT_ID }),
        None,
    )?;
    Ok(())
}

pub fn username(client: &Client, access_token: &str) -> Result<String> {
    let v = get_authed(client, "/users/settings", access_token)?;
    v.pointer("/user/username")
        .and_then(Value::as_str)
        .map(String::from)
        .context("Trakt username")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_token_response() {
        let v: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/trakt/token.json")).unwrap();
        let t = parse_tokens(&v).unwrap();
        assert_eq!(t.expires_at, 1_487_889_741 + 7200);
        assert!(t.access_token.starts_with("dbaf"));
        assert!(t.refresh_token.starts_with("76ba"));
        assert_eq!(t.username, None);
    }

    #[test]
    fn debug_hides_tokens() {
        let v: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/trakt/token.json")).unwrap();
        let shown = format!("{:?}", parse_tokens(&v).unwrap());
        assert!(!shown.contains("dbaf") && !shown.contains("76ba"));
    }

    #[test]
    fn rejects_incomplete_token_response() {
        assert_eq!(parse_tokens(&json!({"access_token": "a"})), None);
        assert_eq!(parse_tokens(&json!({})), None);
    }
}
