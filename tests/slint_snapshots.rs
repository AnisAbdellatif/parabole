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
use iptv_player::slint_ui::{
    AppWindow, BrowseApi, Core, DialogApi, PaletteApi, PlayerApi, StreamItem, Theme, ToastItem,
    build, fonts,
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
        self.win
            .global::<DialogApi>()
            .invoke_open_dialog(name.into());
        self.p.pump(2);
        self.shot(name);
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
        api.set_meta("News".into());
        self.shot("player-chrome");
        self.check_control_alignment();
        let card = |name: &str, n: &str, playing| StreamItem {
            name: name.into(),
            meta: n.into(),
            initials: name[..2].to_uppercase().into(),
            playing,
            ..Default::default()
        };
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
    s.player_screens();
    s.toast_screen();
    let failures = std::mem::take(&mut s.failures);
    drop(s);
    slint_seed::cleanup(&dir);
    assert!(
        failures.is_empty(),
        "snapshot failures:\n{}",
        failures.join("\n")
    );
}
