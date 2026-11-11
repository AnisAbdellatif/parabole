use super::App;
use crate::model::{Season, SourceId};
use crate::worker::Command;

impl App {
    pub(super) fn series_loaded(
        &mut self,
        source: SourceId,
        key: &str,
        result: Result<Vec<Season>, String>,
    ) {
        let Some(s) = self
            .series
            .as_mut()
            .filter(|s| s.source == source && s.key == key)
        else {
            return;
        };
        s.loading = false;
        match result {
            Ok(seasons) => s.seasons = seasons,
            Err(e) => {
                self.series = None;
                self.toast(format!("Could not load series: {e}"));
            }
        }
    }

    pub(super) fn series_ui(&mut self, ui: &mut egui::Ui) {
        let Some(s) = self.series.as_mut() else {
            return;
        };
        let mut back = false;
        let mut play = None;
        ui.horizontal(|ui| {
            back = ui.button("← Back").clicked();
            ui.heading(&s.title);
        });
        if s.loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading episodes…");
            });
        } else if s.seasons.is_empty() {
            ui.weak("No episodes found.");
        } else {
            s.season = s.season.min(s.seasons.len() - 1);
            let name = |se: &Season| {
                if se.name.is_empty() {
                    format!("Season {}", se.number)
                } else {
                    se.name.clone()
                }
            };
            egui::ComboBox::from_id_salt("season")
                .selected_text(name(&s.seasons[s.season]))
                .show_ui(ui, |ui| {
                    for (i, se) in s.seasons.iter().enumerate() {
                        ui.selectable_value(&mut s.season, i, name(se));
                    }
                });
            let season = &s.seasons[s.season];
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    for ep in &season.episodes {
                        let label = match ep.episode_num {
                            Some(n) => format!("E{n}  {}", ep.title),
                            None => ep.title.clone(),
                        };
                        if ui.selectable_label(false, label).double_clicked() {
                            let title = format!("{} S{} {}", s.title, season.number, ep.title);
                            play = Some((s.source, ep.clone(), title));
                        }
                    }
                });
        }
        if let Some((source, episode, title)) = play {
            self.worker.send(Command::PlayEpisode {
                profile: self.profile(),
                source,
                episode,
                title,
                kind: None,
            });
        }
        if back {
            self.series = None;
        }
    }
}
