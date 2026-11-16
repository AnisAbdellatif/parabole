#![cfg(feature = "slint-ui")]
//! Headless snapshot tests: the real window on the software renderer, compared with goldens.
#[path = "common/ink.rs"]
mod ink;
#[path = "common/slint_seed.rs"]
mod slint_seed;
#[path = "common/snapshot.rs"]
mod snapshot;
#[path = "common/sw_platform.rs"]
mod sw_platform;

use i_slint_backend_testing::ElementHandle;
use parabole::slint_ui::{
    AppWindow, BrowseApi, Core, DialogApi, EpisodeItem, FieldItem, InfoView, PaletteApi, PlayerApi,
    SportsApi, SportsChannel, SportsRow, StreamItem, Theme, ToastItem, build, fonts,
};
use slint::platform::{Key, WindowEvent};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use sw_platform::{SCALE, SwPlatform};

struct Shots {
    p: SwPlatform,
    win: AppWindow,
    failures: Vec<String>,
}

impl Shots {
    fn key(&self, text: impl Into<SharedString> + Clone) {
        let w = self.win.window();
        w.dispatch_event(WindowEvent::KeyPressed {
            text: text.clone().into(),
        });
        w.dispatch_event(WindowEvent::KeyReleased { text: text.into() });
    }

    fn shot(&mut self, name: &str) {
        self.p.pump(12);
        if let Err(e) = snapshot::check(name, &self.p.render()) {
            self.failures.push(e);
        }
    }

    fn open_dialog(&mut self, name: &str) {
        self.dialog_view(name, name, |_| {});
    }

    /// Opens dialog `name`, applies `set` (for example a tab), then shoots it as `shot`.
    fn dialog_view(&mut self, name: &str, shot: &str, set: impl Fn(&DialogApi)) {
        self.win
            .global::<DialogApi>()
            .invoke_open_dialog(name.into());
        self.p.pump(2);
        set(&self.win.global::<DialogApi>());
        self.shot(shot);
        self.win.global::<DialogApi>().invoke_close();
        self.p.pump(2);
    }

    fn streams(&self) -> usize {
        self.win.global::<Core>().get_streams().row_count()
    }

    fn live_screens(&mut self) {
        self.shot("live-dark");
        self.win.global::<Theme>().set_mode(2);
        self.shot("live-light");
        self.win.global::<Theme>().set_mode(0);
        self.win.global::<Core>().invoke_tab_selected(1);
        self.p.pump_until("movies loaded", || self.streams() == 6);
        self.shot("movies-wall");
        self.win.global::<Core>().invoke_tab_selected(0);
        self.p.pump_until("live reloaded", || self.streams() == 6);
    }

    fn palette_screen(&mut self) {
        self.win.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Control.into(),
        });
        self.key("k");
        self.win.window().dispatch_event(WindowEvent::KeyReleased {
            text: Key::Control.into(),
        });
        let win = self.win.clone_strong();
        let api = win.global::<PaletteApi>();
        self.p.pump_until("palette open", || api.get_open());
        self.p.pump(2);
        for c in "chan".chars() {
            self.key(c.to_string());
        }
        self.p.pump_until("palette results", || {
            api.get_items().iter().any(|i| i.kind == 0)
        });
        self.shot("palette-chan");
        api.invoke_close();
        self.p.pump(2);
    }

    fn player_screens(&mut self) {
        let win = self.win.clone_strong();
        let api = win.global::<PlayerApi>();
        api.set_active(true);
        api.set_paused(true);
        api.set_is_live(true);
        api.set_title("Channel One".into());
        api.set_tag("LIVE".into());
        api.set_tech("1080p · 5.1".into());
        api.set_meta("Live channel".into());
        let card = |name: &str, n: &str, playing| StreamItem {
            name: name.into(),
            meta: n.into(),
            initials: name[..2].to_uppercase().into(),
            playing,
            ..Default::default()
        };
        let names = [
            "World News",
            "Channel Two",
            "Channel One",
            "Daily Brief",
            "Market Watch",
            "Sports Extra",
            "Movie Classics",
            "Kids Zone",
            "Weather Now",
        ];
        let pane: Vec<_> = names
            .iter()
            .enumerate()
            .map(|(i, n)| card(n, &(i + 1).to_string(), i == 2))
            .collect();
        api.set_pane_items(ModelRc::new(VecModel::from(pane)));
        api.set_pane_current(2);
        self.shot("player-chrome");
        self.check_control_alignment();
        let items = vec![
            card("World News", "1", false),
            card("Channel Two", "2", false),
            card("Channel One", "3", true),
            card("Daily Brief", "4", false),
            card("Market Watch", "5", false),
        ];
        api.set_zap_items(ModelRc::new(VecModel::from(items)));
        api.set_zap_open(true);
        self.shot("zap-carousel");
        api.set_fullscreen(true);
        self.shot("zap-fullscreen");
        api.set_fullscreen(false);
        api.set_mini(true);
        self.shot("zap-mini");
        api.set_mini(false);
        api.set_zap_open(false);
        api.set_active(false);
        self.p.pump(2);
    }

    fn series_screen(&mut self) {
        let win = self.win.clone_strong();
        let api = win.global::<BrowseApi>();
        let model = api.get_episodes();
        let eps = model
            .as_any()
            .downcast_ref::<VecModel<EpisodeItem>>()
            .expect("episodes is a VecModel");
        let ep = |n: i32, progress: f32, watched: bool| EpisodeItem {
            id: n.to_string().into(),
            num: format!("E{n}").into(),
            title: format!("Episode {n}").into(),
            progress,
            watched,
        };
        eps.set_vec(vec![ep(1, 0.0, true), ep(2, 0.4, false), ep(3, 0.0, false)]);
        api.set_series_title("Fixture Show".into());
        api.set_season_label("Season 1".into());
        api.set_episode_cursor(1);
        api.set_series_open(true);
        self.shot("series-episodes");
        api.set_series_open(false);
        eps.set_vec(Vec::new());
        self.p.pump(2);
    }

    /// Long wrapped fields must stay reachable, not cropped by the card.
    fn details_screen(&mut self) {
        let win = self.win.clone_strong();
        let api = win.global::<BrowseApi>();
        let field = |label: &str, value: &str| FieldItem {
            label: label.into(),
            value: value.into(),
        };
        let cast = "Jake Johnson, Mary Steenburgen, Ed Harris, Aaron Chen, Patton Oswalt, Cleo King, Lynne Marie Stewart, Chloe Fineman";
        let plot = "A washed-up tennis pro and his estranged brother team up for a pickleball tournament, where old rivalries, new friends and a lot of bad shots decide whether they can finally work together.";
        api.set_details_title("4K-A+ - The Dink (2026)".into());
        api.set_details(InfoView {
            state: 2,
            fields: ModelRc::new(VecModel::from(vec![
                field("Rating", "★ 6.5 IMDb · 12K votes"),
                field("Genre", "Comedy"),
                field("Released", "2026-07-23"),
                field("Duration", "1h 42m"),
                field("Director", "Josh Greenbaum"),
                field("Cast", cast),
                field("Plot", plot),
            ])),
            ..Default::default()
        });
        api.set_details_open(true);
        self.shot("details");
        api.set_details_open(false);
        self.p.pump(2);
    }

    fn toast_screen(&mut self) {
        let win = self.win.clone_strong();
        let core = win.global::<Core>();
        let item = ToastItem {
            id: 1,
            text: "Refreshed Fixture: 24 items".into(),
        };
        core.set_toasts(ModelRc::new(VecModel::from(vec![item])));
        self.shot("toast");
    }

    fn sports_screen(&mut self) {
        let win = self.win.clone_strong();
        let row = |section: &str,
                   time: &str,
                   home: &str,
                   away: &str,
                   score: (&str, &str),
                   langs: &str| {
            SportsRow {
                key: home.into(),
                section: section.into(),
                time: time.into(),
                live: !score.0.is_empty(),
                home: home.into(),
                away: away.into(),
                home_score: score.0.into(),
                away_score: score.1.into(),
                langs: langs.into(),
            }
        };
        let ch = |header: bool, text: &str, channel: &str| SportsChannel {
            header,
            text: text.into(),
            channel: channel.into(),
        };
        let api = win.global::<SportsApi>();
        api.set_rows(ModelRc::new(VecModel::from(vec![
            row(
                "Champions League",
                "67'",
                "Real Madrid",
                "Inter",
                ("1", "1"),
                "EN ES FR +3",
            ),
            row(
                "Nations League",
                "20:45",
                "Germany",
                "Netherlands",
                ("", ""),
                "DE EN FR AR +4",
            ),
            row("", "21:00", "France", "Italy", ("", ""), "FR IT"),
            row(
                "Formula 1",
                "05:00",
                "Las Vegas Grand Prix",
                "",
                ("", ""),
                "ES",
            ),
        ])));
        let names =
            ["Football", "Basketball", "Tennis", "F1", "WWE", "UFC"].map(SharedString::from);
        api.set_sport_names(ModelRc::new(VecModel::from(names.to_vec())));
        api.set_sport(0);
        api.set_live_count(1);
        api.set_day_label("Today".into());
        api.set_can_next(true);
        api.set_cursor(1);
        api.set_detail_title("Germany – Netherlands".into());
        api.set_detail_sub("Nations League · Mon 16 Nov, 20:45".into());
        api.set_channels(ModelRc::new(VecModel::from(vec![
            ch(true, "English", ""),
            ch(false, "Amazon Prime PPV (GB)", "Amazon Prime PPV"),
            ch(false, "Fox Sports 1 (US)", "Fox Sports 1"),
            ch(true, "German", ""),
            ch(false, "ARD Das Erste (DE)", "ARD Das Erste"),
            ch(true, "Arabic", ""),
            ch(false, "beIN Sport 1 HD (SA)", "beIN Sport 1 HD"),
        ])));
        api.set_status("Updated 14:32 · LiveScore unavailable".into());
        api.set_empty_text("".into());
        let core = win.global::<Core>();
        core.set_toasts(ModelRc::new(VecModel::default()));
        core.set_sports_enabled(true);
        core.set_tab(3);
        self.p.pump(2);
        self.shot("sports");
        core.set_tab(0);
        core.set_sports_enabled(false);
        api.set_cursor(-1);
        self.p.pump(2);
    }

    fn rect(&self, id: &str, nth: usize) -> ink::Rect {
        let el = ElementHandle::find_by_element_id(&self.win, id)
            .nth(nth)
            .unwrap_or_else(|| panic!("no element {id} #{nth}"));
        let (pos, size) = (el.absolute_position(), el.size());
        ink::Rect {
            x: pos.x,
            y: pos.y,
            w: size.width,
            h: size.height,
        }
    }

    fn check_control_alignment(&mut self) {
        let img = self.p.render();
        let ids = ["pause", "stop", "mute", "info", "mini", "fullscreen"];
        let centres: Vec<(&str, i64)> = ids
            .iter()
            .map(|n| {
                let r = self.rect(&format!("PlayerControls::{n}-btn"), 0);
                let rows = ink::ink_rows(&img, &r, SCALE, 60)
                    .unwrap_or_else(|| panic!("no ink for control {n}"));
                (*n, ink::centre2(rows))
            })
            .collect();
        let (lo, hi) = centres.iter().fold((i64::MAX, i64::MIN), |(l, h), (_, c)| {
            (l.min(*c), h.max(*c))
        });
        // centre2 is doubled: 2 units = 1 physical px
        if hi - lo > 2 {
            self.failures.push(format!(
                "control row misaligned (doubled centres): {centres:?}"
            ));
        }
    }

    fn check_sidebar_row(&mut self) {
        let img = self.p.render();
        let rows: Vec<_> = self
            .win
            .global::<BrowseApi>()
            .get_side_rows()
            .iter()
            .collect();
        let clickable: Vec<_> = rows.iter().filter(|r| r.kind != 0 && r.kind != 4).collect();
        let at = clickable
            .iter()
            .position(|r| r.kind == 3)
            .expect("a category row");
        let row = self.rect("SideList::ta", at);
        let badge = ink::Rect {
            x: row.x + row.w - 60.0,
            y: row.y,
            w: 52.0,
            h: row.h,
        };
        let (Some(bx), Some(digits)) = (
            ink::ink_rows(&img, &badge, SCALE, 4),
            ink::ink_rows(&img, &badge, SCALE, 80),
        ) else {
            self.failures.push("sidebar row: badge not found".into());
            return;
        };
        let row_mid = ink::centre2((
            (row.y * SCALE).round() as u32,
            ((row.y + row.h) * SCALE).round() as u32 - 1,
        ));
        for (what, c) in [
            ("badge box", ink::centre2(bx)),
            ("badge digits", ink::centre2(digits)),
        ] {
            if (c - row_mid).abs() > 2 {
                self.failures.push(format!(
                    "sidebar row: {what} centre {c} vs row centre {row_mid} (doubled px)"
                ));
            }
        }
    }
}

#[test]
fn snapshots() {
    let dir = slint_seed::scratch_dir("snapshots");
    slint_seed::isolate_home(&dir);
    let db = slint_seed::seed(&dir);
    slint_seed::add_profile(&db, "Kids Zone");
    slint_seed::write_state(&db, r#"{"appearance":{"reduce_motion":true}}"#);
    let p = SwPlatform::install();
    fonts::install_fallback();
    let win = build(&db).expect("build window");
    #[cfg(target_os = "macos")]
    assert_eq!(win.global::<Theme>().get_font(), ".SF NS");
    // Keep goldens portable rather than dependent on macOS system fonts.
    win.global::<Theme>().set_font("Inter".into());
    win.show().expect("show window");
    p.resize();
    win.window()
        .dispatch_event(WindowEvent::WindowActiveChanged(true));
    let mut s = Shots {
        p,
        win,
        failures: Vec::new(),
    };
    s.p.pump_until("streams loaded", || s.streams() > 0);
    s.live_screens();
    s.check_sidebar_row();
    s.palette_screen();
    for name in ["appearance", "settings", "sources", "profiles"] {
        s.open_dialog(name);
    }
    s.dialog_view("sources", "sources-form", |d| d.set_form_open(true));
    s.dialog_view("settings", "settings-players", |d| d.set_settings_tab(1));
    s.series_screen();
    s.details_screen();
    s.player_screens();
    s.toast_screen();
    s.sports_screen();
    let failures = std::mem::take(&mut s.failures);
    drop(s);
    slint_seed::cleanup(&dir);
    assert!(
        failures.is_empty(),
        "snapshot failures:\n{}",
        failures.join("\n")
    );
}
