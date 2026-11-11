use super::credentials;
use crate::model::Source;
use crate::sources::http;
use anyhow::{Context, Result};
use reqwest::blocking::Client as Http;
use serde_json::Value;
use url::Url;

pub struct Client {
    http: Http,
    user_agent: Option<String>,
    endpoint: String,
    username: String,
    password: String,
}

impl Client {
    pub fn new(source: &Source) -> Result<Self> {
        let (username, password) = credentials(source)?;
        Ok(Self {
            user_agent: source.user_agent.clone(),
            http: http::client(source.user_agent.as_deref())?,
            endpoint: format!("{}/player_api.php", source.url.trim_end_matches('/')),
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    pub fn call(&self, action: Option<&str>, extra: &[(&str, &str)]) -> Result<Value> {
        let mut url = Url::parse(&self.endpoint).context("invalid server URL")?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("username", &self.username)
                .append_pair("password", &self.password);
            if let Some(action) = action {
                q.append_pair("action", action);
            }
            q.extend_pairs(extra);
        }
        let body = http::get(&self.http, url.as_str(), self.user_agent.as_deref())?
            .bytes()
            .map_err(reqwest::Error::without_url)
            .context("read response")?;
        serde_json::from_slice(&body).context("server did not return valid JSON")
    }

    /// Lists come back as `false`/`null` when empty; treat anything but an array as empty.
    pub fn list(&self, action: &str) -> Result<Vec<Value>> {
        match self.call(Some(action), &[])? {
            Value::Array(items) => Ok(items),
            _ => Ok(Vec::new()),
        }
    }
}
