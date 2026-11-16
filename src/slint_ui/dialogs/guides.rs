//! Settings → TV guides tab: list, enable, order, add, remove, refresh. Changes apply immediately.
use super::{AppWindow, DialogApi};
use crate::epg::catalog;
use crate::model::Guide;
use crate::slint_ui::GuideRowItem;
use crate::slint_ui::epg::format::now_secs;
use crate::slint_ui::state::{SlintApp, with_app};
use crate::worker::Command;
use slint::{ComponentHandle, ModelRc, VecModel};
use std::collections::HashSet;

/// Host of a guide URL; the full URL may carry tokens and is never shown.
fn host(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()?
        .host_str()
        .map(|h| h.trim_start_matches("www.").to_string())
}

fn ago(secs: i64) -> String {
    match secs.max(0) {
        s if s < 60 => "just now".into(),
        s if s < 3600 => format!("{} min ago", s / 60),
        s if s < 86400 => format!("{} h ago", s / 3600),
        s => format!("{} d ago", s / 86400),
    }
}

/// Network errors can quote the guide URL; hide anything that looks like one.
fn scrub(err: &str) -> String {
    let words: Vec<&str> = err
        .split_whitespace()
        .map(|w| if w.contains("://") { "<url>" } else { w })
        .collect();
    words.join(" ").chars().take(80).collect()
}

fn detail(g: &Guide, covered: usize, now: i64) -> String {
    let base = match (&g.source_id, &g.url) {
        (Some(_), _) | (None, None) => "From your provider".to_string(),
        (None, Some(u)) => host(u).unwrap_or_else(|| "custom guide".into()),
    };
    let channels = if covered == 1 { "channel" } else { "channels" };
    let status = match (&g.last_error, g.refreshed_at) {
        (Some(e), _) => format!("failed: {}", scrub(e)),
        (None, Some(t)) => format!("updated {}", ago(now - t)),
        (None, None) => "never updated".into(),
    };
    format!("{base} · covers {covered} {channels} · {status}")
}

impl SlintApp {
    pub(super) fn push_guides(&self) {
        let Some(w) = self.win.upgrade() else { return };
        let guides = self.db.epg_guides().unwrap_or_default();
        let cover = self.db.epg_coverage().unwrap_or_default();
        let now = now_secs();
        let rows: Vec<GuideRowItem> = guides
            .iter()
            .map(|g| GuideRowItem {
                id: g.id as i32,
                name: g.name.as_str().into(),
                detail: detail(g, cover.get(&g.id).copied().unwrap_or(0), now).into(),
                enabled: g.enabled,
                removable: g.source_id.is_none(),
                error: g.last_error.is_some(),
            })
            .collect();
        let api = w.global::<DialogApi>();
        api.set_guides_have_added(rows.iter().any(|r| r.removable));
        api.set_guides(ModelRc::new(VecModel::from(rows)));
        let n = self.suggested_guides().map_or(0, |s| s.len());
        api.set_guide_suggest_count(n as i32);
    }

    /// Catalog guides for the playlist's countries that are not guides yet, as (url, name).
    fn suggested_guides(&self) -> anyhow::Result<Vec<(&'static str, &'static str)>> {
        let have: HashSet<String> = self
            .db
            .epg_guides()?
            .into_iter()
            .filter_map(|g| g.url)
            .collect();
        let list = catalog::suggested(&self.db.epg_stream_keys()?);
        Ok(list
            .into_iter()
            .filter(|g| !have.contains(g.url))
            .map(|g| (g.url, g.name))
            .collect())
    }

    fn add_suggested(&mut self) -> anyhow::Result<()> {
        let list = self.suggested_guides()?;
        for (url, name) in &list {
            let id = self.db.add_epg_guide(url, name)?;
            self.worker.send(Command::RefreshEpg(id));
        }
        self.toast(format!("Added {} TV guides", list.len()));
        Ok(())
    }

    fn guide_action(&mut self, f: impl FnOnce(&mut SlintApp) -> anyhow::Result<()>) {
        if let Err(e) = f(self) {
            self.toast(format!("{e:#}"));
        }
        self.push_guides();
    }

    fn add_guide(&mut self, url: &str) -> anyhow::Result<()> {
        let url = url.trim();
        let name = host(url).unwrap_or_else(|| "Guide".into());
        let id = self.db.add_epg_guide(url, &name)?;
        self.worker.send(Command::RefreshEpg(id));
        if let Some(w) = self.win.upgrade() {
            w.global::<DialogApi>().set_guide_url("".into());
        }
        Ok(())
    }

    fn toggle_guide(&mut self, id: i64) -> anyhow::Result<()> {
        let Some(g) = self.db.epg_guides()?.into_iter().find(|g| g.id == id) else {
            return Ok(());
        };
        self.db.set_epg_guide_enabled(id, !g.enabled)?;
        self.worker.send(Command::RelinkEpg);
        if !g.enabled {
            self.worker.send(Command::RefreshEpg(id));
        }
        Ok(())
    }
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<DialogApi>();
    api.on_guide_added(|url| {
        with_app(|a| a.guide_action(|a| a.add_guide(&url)));
    });
    api.on_guide_toggled(|id| {
        with_app(|a| a.guide_action(|a| a.toggle_guide(id.into())));
    });
    api.on_guide_moved(|id, d| {
        with_app(|a| {
            a.guide_action(|a| {
                a.db.move_epg_guide(id.into(), d.into())?;
                a.worker.send(Command::RelinkEpg);
                Ok(())
            })
        });
    });
    api.on_guide_removed(|id| {
        with_app(|a| {
            a.guide_action(|a| {
                a.db.delete_epg_guide(id.into())?;
                a.worker.send(Command::RelinkEpg);
                Ok(())
            })
        });
    });
    api.on_guides_add_suggested(|| {
        with_app(|a| a.guide_action(|a| a.add_suggested()));
    });
    api.on_guides_remove_added(|| {
        with_app(|a| {
            a.guide_action(|a| {
                a.db.delete_added_epg_guides()?;
                a.worker.send(Command::RelinkEpg);
                Ok(())
            })
        });
    });
    api.on_guides_refresh_all(|| {
        with_app(|a| {
            a.guide_action(|a| {
                for g in a.db.epg_guides()?.into_iter().filter(|g| g.enabled) {
                    a.worker.send(Command::RefreshEpg(g.id));
                }
                Ok(())
            })
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guide(source: Option<i64>, url: Option<&str>) -> Guide {
        Guide {
            id: 1,
            source_id: source,
            url: url.map(Into::into),
            name: "g".into(),
            enabled: true,
            position: 0,
            refreshed_at: None,
            last_error: None,
        }
    }

    #[test]
    fn host_only() {
        assert_eq!(
            host("https://www.x.org/a/b?token=1").as_deref(),
            Some("x.org")
        );
        assert_eq!(host("nonsense"), None);
    }

    #[test]
    fn provider_detail() {
        let mut g = guide(Some(1), None);
        g.refreshed_at = Some(1000);
        assert_eq!(
            detail(&g, 5, 1000 + 3 * 3600),
            "From your provider · covers 5 channels · updated 3 h ago"
        );
    }

    #[test]
    fn user_detail_hides_url() {
        let mut g = guide(None, Some("https://h.com/g.xml?k=secret"));
        assert_eq!(detail(&g, 1, 0), "h.com · covers 1 channel · never updated");
        g.last_error = Some("error for url (https://h.com/g.xml?k=secret): 404".into());
        let d = detail(&g, 0, 0);
        assert!(d.contains("failed:") && !d.contains("secret"), "{d}");
    }
}
