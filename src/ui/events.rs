use super::App;
use crate::worker::Event;

impl App {
    pub(super) fn pump_events(&mut self, ctx: &egui::Context) {
        while let Ok(ev) = self.worker.rx.try_recv() {
            match ev {
                Event::SourceChanged => self.reload_sources(),
                Event::RefreshStarted(id) => {
                    self.refreshing.insert(id);
                }
                Event::RefreshDone(id, res) => {
                    self.refreshing.remove(&id);
                    if let Err(e) = res {
                        self.toast(format!("Refresh failed: {e}"));
                    }
                    self.reload_sources();
                }
                Event::AccountChecked(id, res) => {
                    if let Some(d) = self.sources_dialog.as_mut() {
                        d.account_checked(id);
                    }
                    match res {
                        Ok(()) => self.reload_accounts(),
                        Err(e) => self.toast(format!("Account check failed: {e}")),
                    }
                }
                Event::TestResult(res) => {
                    if let Some(d) = self.sources_dialog.as_mut() {
                        d.test_finished(res);
                    }
                }
                Event::InfoLoaded {
                    source,
                    media,
                    key,
                    result,
                } => self.info_loaded(source, media, key, result),
                Event::SeriesLoaded {
                    source,
                    key,
                    result,
                } => self.series_loaded(source, &key, result),
                Event::Playing { title, kind, media } => {
                    self.player_ui.last_error = None;
                    self.external_started(ctx, media);
                    self.playing = Some((title, kind));
                }
                Event::PlayEmbedded(play) => self.on_embed_play(*play),
                Event::PlayerStopped => {
                    if !self.embedded_active() {
                        self.playing = None;
                    }
                }
                Event::PlayFailed(msg) => {
                    self.player_ui.last_error = Some(msg.clone());
                    self.toast(msg);
                }
                Event::SearchResults { .. } => {}
                Event::UrlReady(url) => {
                    ctx.copy_text(url);
                    self.toast("Stream URL copied");
                }
                Event::Error(msg) => self.toast(msg),
            }
        }
    }
}
