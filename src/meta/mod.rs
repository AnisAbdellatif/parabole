//! Online details of movies and series: title cleaning, candidate picking and the IMDb client.
//! No DB, no UI.
pub mod imdb;
pub mod pick;
pub mod title;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use std::time::Duration;

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36";

pub fn client() -> Result<Client> {
    Client::builder()
        .user_agent(UA)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .build()
        .context("build http client")
}
