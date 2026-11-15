#![cfg(feature = "slint-ui")]
//! Headless behaviour tests: the real window on the testing backend, no pixels, no OS input.
#[path = "common/slint_seed.rs"]
mod slint_seed;

use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use iptv_player::slint_ui::{
    AppWindow, AppearanceApi, BrowseApi, Core, DialogApi, EpisodeItem, PaletteApi, Theme, build,
    fonts,
};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, Model, SharedString, VecModel};
use std::cell::{Cell, RefCell};
use std::future::poll_fn;
use std::path::PathBuf;
use std::rc::Rc;
use std::task::{Poll, Waker};
use std::time::{Duration, Instant};

async fn settle(ms: u64) {
    let done = Rc::new(Cell::new(false));
    let waker: Rc<RefCell<Option<Waker>>> = Rc::default();
    let (d, w) = (done.clone(), waker.clone());
    slint::Timer::single_shot(Duration::from_millis(ms), move || {
        d.set(true);
        if let Some(w) = w.borrow_mut().take() {
            w.wake();
        }
    });
    poll_fn(|cx| {
        if done.get() {
            return Poll::Ready(());
        }
        *waker.borrow_mut() = Some(cx.waker().clone());
        Poll::Pending
    })
    .await
}

async fn wait_until(what: &str, cond: impl Fn() -> bool) {
    let start = Instant::now();
    while !cond() {
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "timed out waiting for: {what}"
        );
        settle(10).await;
    }
}

fn key_down(win: &AppWindow, text: impl Into<SharedString>) {
    win.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.into() });
}

fn key_up(win: &AppWindow, text: impl Into<SharedString>) {
    win.window()
        .dispatch_event(WindowEvent::KeyReleased { text: text.into() });
}

fn press(win: &AppWindow, text: impl Into<SharedString> + Clone) {
    key_down(win, text.clone());
    key_up(win, text);
}

fn chord(win: &AppWindow, modifier: Key, text: &str) {
    key_down(win, modifier);
    press(win, text);
    key_up(win, modifier);
}

fn type_text(win: &AppWindow, text: &str) {
    for c in text.chars() {
        press(win, c.to_string());
    }
}

async fn click(win: &AppWindow, id: &str, nth: usize) {
    let found: Vec<_> = ElementHandle::find_by_element_id(win, id).collect();
    let el = found
        .get(nth)
        .unwrap_or_else(|| panic!("no element {id} #{nth} (found {})", found.len()));
    el.single_click(PointerEventButton::Left).await;
    settle(30).await;
}

fn state_json(dir: &std::path::Path) -> serde_json::Value {
    let bytes = std::fs::read(dir.join("slint-ui.json")).expect("state file written");
    serde_json::from_slice(&bytes).expect("state file is json")
}

async fn open_palette(win: &AppWindow, case: &str) {
    chord(win, Key::Control, "k");
    wait_until(&format!("{case}: palette open"), || {
        win.global::<PaletteApi>().get_open()
    })
    .await;
    settle(50).await;
}

async fn startup(win: &AppWindow) {
    let core = win.global::<Core>();
    wait_until("startup: streams loaded", || {
        core.get_streams().row_count() > 0
    })
    .await;
    assert!(
        core.get_selected_index() >= 0 && !core.get_selected_id().is_empty(),
        "startup: first category selected"
    );
}

async fn sources_dialog(win: &AppWindow) {
    let api = win.global::<DialogApi>();
    api.invoke_open_dialog("sources".into());
    wait_until("sources: dialog open", || api.get_open() == "sources").await;
    assert_eq!(
        api.get_sources().row_count(),
        1,
        "sources: list filled on open"
    );
    assert!(!api.get_form_open(), "sources: list view first");
    settle(350).await;
    click(win, "SourcesDialog::add", 0).await;
    wait_until("sources: Add source opens the form", || api.get_form_open()).await;
    settle(50).await;
    press(win, Key::Escape);
    wait_until("sources: Esc returns to the list", || !api.get_form_open()).await;
    assert_eq!(
        api.get_open(),
        "sources",
        "sources: Esc keeps the dialog open"
    );
    api.invoke_close();
    wait_until("sources: closed", || api.get_open().is_empty()).await;
}

async fn sidebar(win: &AppWindow) {
    let core = win.global::<Core>();
    let first = core.get_selected_id();
    let titles = |win: &AppWindow| -> Vec<String> {
        let rows = win.global::<Core>().get_streams();
        rows.iter().map(|r| r.name.to_string()).collect()
    };
    let before = titles(win);
    let rows: Vec<_> = win.global::<BrowseApi>().get_side_rows().iter().collect();
    // one `SideList::ta` exists per row that is not a header
    let clickable: Vec<_> = rows.iter().filter(|r| r.kind != 0 && r.kind != 4).collect();
    let second = clickable
        .iter()
        .position(|r| r.kind == 3 && r.id != first)
        .expect("sidebar: a second category row");
    click(win, "SideList::ta", second).await;
    wait_until("sidebar: selection changes", || {
        core.get_selected_id() != first
    })
    .await;
    wait_until("sidebar: stream rows change", || titles(win) != before).await;
}

async fn sidebar_filter(win: &AppWindow) {
    let api = win.global::<BrowseApi>();
    let rows = || api.get_side_rows().iter().collect::<Vec<_>>();
    let all = rows().len();
    let label = rows()
        .iter()
        .rev()
        .find(|r| r.kind == 3)
        .map(|r| r.label.to_string())
        .expect("sidebar filter: a category row");
    click(win, "SideFilter::input", 0).await;
    type_text(win, &label.to_uppercase());
    wait_until("sidebar filter: rows reduced", || {
        let r = rows();
        r.len() < all && r.iter().any(|r| r.label == label.as_str())
    })
    .await;
    press(win, Key::Escape);
    wait_until("sidebar filter: Esc restores rows", || rows().len() == all).await;
    assert!(api.get_side_filter().is_empty(), "sidebar filter: cleared");
}

async fn palette(win: &AppWindow) {
    let api = win.global::<PaletteApi>();
    open_palette(win, "palette").await;
    type_text(win, "chan");
    wait_until("palette: worker results for 'chan'", || {
        api.get_items().iter().any(|i| i.kind == 0)
    })
    .await;
    press(win, Key::DownArrow);
    assert_eq!(api.get_cursor(), 1, "palette: Down moves the cursor");
    press(win, Key::Escape);
    wait_until("palette: Esc closes", || !api.get_open()).await;

    // Enter on a stream would start a player; a category result exercises activation safely.
    let before = win.global::<Core>().get_selected_id();
    open_palette(win, "palette").await;
    type_text(win, "kids");
    wait_until("palette: category result for 'kids'", || {
        api.get_items().iter().any(|i| i.kind == 3)
    })
    .await;
    let at = api
        .get_items()
        .iter()
        .position(|i| i.kind == 3)
        .unwrap_or(0);
    for _ in 0..at {
        press(win, Key::DownArrow);
    }
    press(win, Key::Return);
    wait_until("palette: Enter closes it", || !api.get_open()).await;
    wait_until("palette: category selected", || {
        win.global::<Core>().get_selected_id() != before
    })
    .await;
}

async fn appearance(win: &AppWindow, dir: &std::path::Path) {
    win.global::<DialogApi>()
        .invoke_open_dialog("appearance".into());
    wait_until("appearance: settings open on its tab", || {
        let api = win.global::<DialogApi>();
        api.get_open() == "settings" && api.get_settings_tab() == 2
    })
    .await;
    settle(350).await;
    let theme = win.global::<Theme>();
    let accent = theme.get_accent();
    assert_eq!(theme.get_motion(), 180, "appearance: motion starts on");
    click(win, "AppearancePane::swatch", 1).await;
    assert_ne!(
        theme.get_accent(),
        accent,
        "appearance: accent swatch applies"
    );
    click(win, "AppearancePane::motion-switch", 0).await;
    assert_eq!(theme.get_motion(), 0, "appearance: reduce motion applies");
    let saved = state_json(dir);
    assert_eq!(
        saved["appearance"]["accent"], 1,
        "appearance: accent persisted"
    );
    assert_eq!(
        saved["appearance"]["reduce_motion"], true,
        "appearance: reduce_motion persisted"
    );
    slider_drag(win, dir).await;
    press(win, Key::Escape);
    wait_until("appearance: Esc closes", || {
        win.global::<DialogApi>().get_open().is_empty()
    })
    .await;
}

async fn slider_drag(win: &AppWindow, dir: &std::path::Path) {
    let el = ElementHandle::find_by_element_id(win, "AppearancePane::radius-slider")
        .next()
        .expect("radius slider");
    let (pos, size) = (el.absolute_position(), el.size());
    let at = |f: f32| {
        LogicalPosition::new(
            pos.x + 7.0 + (size.width - 14.0) * f,
            pos.y + size.height / 2.0,
        )
    };
    let before = std::fs::read(dir.join("slint-ui.json")).expect("state file");
    let w = win.window();
    w.dispatch_event(WindowEvent::PointerMoved { position: at(0.2) });
    w.dispatch_event(WindowEvent::PointerPressed {
        position: at(0.2),
        button: PointerEventButton::Left,
    });
    for f in [0.4, 0.6, 0.8] {
        w.dispatch_event(WindowEvent::PointerMoved { position: at(f) });
        settle(10).await;
    }
    let radius = win.global::<AppearanceApi>().get_radius();
    assert!((radius - 1.3).abs() < 0.02, "slider: live value {radius}");
    assert!(
        (win.global::<Theme>().get_radius_scale() - radius).abs() < 1e-3,
        "slider: applied live"
    );
    let during = std::fs::read(dir.join("slint-ui.json")).expect("state file");
    assert_eq!(before, during, "slider: no save while dragging");
    w.dispatch_event(WindowEvent::PointerReleased {
        position: at(0.8),
        button: PointerEventButton::Left,
    });
    settle(30).await;
    let saved = state_json(dir)["appearance"]["radius_scale"]
        .as_f64()
        .expect("radius saved");
    assert!(
        (saved - f64::from(radius)).abs() < 1e-3,
        "slider: saved final value {saved}"
    );
}

async fn dialog_focus(win: &AppWindow) {
    click(win, "ShellRail::b-settings", 0).await;
    wait_until("dialog focus: settings open", || {
        win.global::<DialogApi>().get_open() == "settings"
    })
    .await;
    settle(50).await;
    press(win, Key::Escape);
    wait_until("dialog focus: Esc closes settings", || {
        win.global::<DialogApi>().get_open().is_empty()
    })
    .await;
    settle(50).await;
    open_palette(win, "dialog focus").await;
    press(win, Key::Escape);
    wait_until("dialog focus: palette closes", || {
        !win.global::<PaletteApi>().get_open()
    })
    .await;
}

async fn accessibility(win: &AppWindow) {
    let api = win.global::<DialogApi>();
    let settings = ElementHandle::find_by_accessible_label(win, "Settings")
        .find(|e| e.accessible_role() == Some(AccessibleRole::Button))
        .expect("a11y: rail Settings button");
    settings.invoke_accessible_default_action();
    wait_until("a11y: default action opens settings", || {
        api.get_open() == "settings"
    })
    .await;
    settle(50).await;
    assert!(
        ElementHandle::find_by_accessible_label(win, "Close")
            .next()
            .is_some(),
        "a11y: dialog close button labelled"
    );
    api.invoke_close();
    wait_until("a11y: closed", || api.get_open().is_empty()).await;
}

async fn episode_cursor(win: &AppWindow) {
    let api = win.global::<BrowseApi>();
    let model = api.get_episodes();
    let eps = model
        .as_any()
        .downcast_ref::<VecModel<EpisodeItem>>()
        .expect("episodes is a VecModel");
    eps.set_vec(
        (1..=3)
            .map(|n| EpisodeItem {
                id: n.to_string().into(),
                num: format!("E{n}").into(),
                title: format!("Episode {n}").into(),
                progress: 0.0,
                watched: false,
            })
            .collect::<Vec<_>>(),
    );
    api.set_episode_cursor(0);
    api.set_series_open(true);
    settle(50).await;
    press(win, Key::DownArrow);
    wait_until("episodes: Down moves the cursor", || {
        api.get_episode_cursor() == 1
    })
    .await;
    press(win, Key::Home);
    wait_until("episodes: Home returns to the first", || {
        api.get_episode_cursor() == 0
    })
    .await;
    api.set_series_open(false);
    eps.set_vec(Vec::new());
    settle(30).await;
}

async fn profile_switch(win: &AppWindow) {
    let api = win.global::<PaletteApi>();
    open_palette(win, "profile").await;
    type_text(win, "kids zone");
    wait_until("profile: switch action listed", || {
        api.get_items()
            .iter()
            .any(|i| i.title.contains("Kids Zone"))
    })
    .await;
    let at = api
        .get_items()
        .iter()
        .position(|i| i.title.contains("Kids Zone"))
        .unwrap_or(0);
    for _ in 0..at {
        press(win, Key::DownArrow);
    }
    press(win, Key::Return);
    wait_until("profile: active profile changes", || {
        win.global::<Core>().get_profile_name() == "Kids Zone"
    })
    .await;
}

async fn profile_switch_dialog(win: &AppWindow) {
    let api = win.global::<DialogApi>();
    let before = win.global::<Core>().get_profile_name();
    api.invoke_open_dialog("profiles".into());
    wait_until("profiles dialog: open", || api.get_open() == "profiles").await;
    settle(50).await;
    click(win, "ProfileItem::switch", 0).await;
    wait_until("profiles dialog: Switch changes the profile", || {
        win.global::<Core>().get_profile_name() != before
    })
    .await;
    let name = win.global::<Core>().get_profile_name();
    wait_until("profiles dialog: Active badge moves", || {
        api.get_profile_rows()
            .iter()
            .any(|r| r.active && r.name == name)
    })
    .await;
    assert_eq!(api.get_open(), "profiles", "profiles dialog: stays open");
    api.invoke_close();
    wait_until("profiles dialog: closed", || api.get_open().is_empty()).await;
}

fn scratch() -> PathBuf {
    let dir = slint_seed::scratch_dir("behaviour");
    slint_seed::isolate_home(&dir);
    dir
}

#[test]
fn behaviour() {
    let dir = scratch();
    let db = slint_seed::seed(&dir);
    slint_seed::add_profile(&db, "Kids Zone");
    i_slint_backend_testing::init_integration_test_with_system_time();
    fonts::install_fallback();
    let win = build(&db).expect("build window");
    win.show().expect("show window");
    win.window()
        .dispatch_event(WindowEvent::WindowActiveChanged(true));
    let task = {
        let (win, dir) = (win.clone_strong(), dir.clone());
        async move {
            startup(&win).await;
            sources_dialog(&win).await;
            sidebar(&win).await;
            sidebar_filter(&win).await;
            palette(&win).await;
            appearance(&win, &dir).await;
            dialog_focus(&win).await;
            accessibility(&win).await;
            profile_switch(&win).await;
            profile_switch_dialog(&win).await;
            episode_cursor(&win).await;
            slint::quit_event_loop().ok();
        }
    };
    slint::spawn_local(task).expect("spawn test task");
    slint::run_event_loop().expect("event loop");
    slint_seed::cleanup(&dir);
}
