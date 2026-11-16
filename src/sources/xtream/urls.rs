use super::{credentials, failover};
use crate::model::{MediaKind, PlayInfo, Source};
use anyhow::{Context, Result, anyhow, bail};
use url::Url;

pub fn resolve(source: &Source, info: &PlayInfo) -> Result<String> {
    match info.media {
        MediaKind::Live => {
            let ext = source.output_format.as_deref().unwrap_or("ts");
            build(source, "live", &info.key, ext)
        }
        MediaKind::Movie => {
            let ext = info.container_ext.as_deref().unwrap_or("mp4");
            build(source, "movie", &info.key, ext)
        }
        MediaKind::Series => bail!("select an episode"),
    }
}

pub fn episode(source: &Source, episode_id: &str, container_ext: Option<&str>) -> Result<String> {
    build(source, "series", episode_id, container_ext.unwrap_or("mp4"))
}

fn build(source: &Source, kind: &str, key: &str, ext: &str) -> Result<String> {
    let source = &failover::pick(source);
    let (user, pass) = credentials(source)?;
    let mut url = Url::parse(&source.url).context("invalid server URL")?;
    let file = format!("{key}.{ext}");
    url.path_segments_mut()
        .map_err(|_| anyhow!("invalid server URL"))?
        .pop_if_empty()
        .extend([kind, user, pass, file.as_str()]);
    Ok(url.into())
}

/// The provider's XMLTV guide URL (carries the credentials; never log it).
pub fn xmltv(source: &Source) -> Result<String> {
    let source = &failover::pick(source);
    let (user, pass) = credentials(source)?;
    let mut url = Url::parse(&source.url).context("invalid server URL")?;
    url.path_segments_mut()
        .map_err(|_| anyhow!("invalid server URL"))?
        .pop_if_empty()
        .push("xmltv.php");
    url.query_pairs_mut()
        .append_pair("username", user)
        .append_pair("password", pass);
    Ok(url.into())
}
