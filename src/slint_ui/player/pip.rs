//! Real picture-in-picture: a small frameless always-on-top window at the screen's bottom-right.
//! `AppWindow` binds `always-on-top`/`no-frame` to `PlayerApi.pip`; this only moves and sizes it.
use super::super::{AppWindow, PlayerApi};
use slint::winit_030::WinitWindowAccessor;
use slint::{ComponentHandle, LogicalSize, PhysicalPosition, PhysicalSize};

const SIZE: LogicalSize = LogicalSize::new(480.0, 270.0);
/// Gap to the screen edge or dock, in logical points.
const MARGIN: f32 = 24.0;
// shortcut: fixed margin off macOS; read the OS work area there if a taskbar overlaps
#[cfg(not(target_os = "macos"))]
const EXTRA_INSET: f64 = 56.0;

/// Window geometry to come back to when PiP ends.
#[derive(Clone)]
pub struct Geometry {
    size: PhysicalSize,
    pos: PhysicalPosition,
    maximized: bool,
}

pub fn connect(win: &AppWindow) {
    let weak = win.as_weak();
    win.global::<PlayerApi>().on_pip_drag(move || {
        if let Some(w) = weak.upgrade() {
            w.window().with_winit_window(|ww| {
                let _ = ww.drag_window();
            });
        }
    });
}

pub fn save(w: &AppWindow) -> Geometry {
    Geometry {
        size: w.window().size(),
        pos: w.window().position(),
        maximized: w.window().is_maximized(),
    }
}

pub fn enter(w: &AppWindow) {
    let window = w.window();
    // a maximized window ignores resize requests (X11 window managers), so PiP would stay full size
    window.set_maximized(false);
    window.set_size(SIZE);
    let scale = window.scale_factor();
    let monitor = window
        .with_winit_window(|ww| ww.current_monitor())
        .flatten();
    if let Some(m) = monitor {
        let (mp, ms) = (m.position(), m.size());
        let (right, bottom) = insets(&m);
        let x = mp.x + ms.width as i32
            - ((SIZE.width + MARGIN) * scale + (right * scale as f64) as f32) as i32;
        let y = mp.y + ms.height as i32
            - ((SIZE.height + MARGIN) * scale + (bottom * scale as f64) as f32) as i32;
        window.set_position(PhysicalPosition::new(x, y));
    }
}

pub fn restore(w: &AppWindow, g: &Geometry) {
    w.window().set_size(g.size);
    w.window().set_position(g.pos);
    w.window().set_maximized(g.maximized);
}

/// Right and bottom space (logical points) the dock or taskbar takes from the monitor.
#[cfg(target_os = "macos")]
fn insets(m: &slint::winit_030::winit::monitor::MonitorHandle) -> (f64, f64) {
    work_area_insets(m).unwrap_or((0.0, 0.0))
}

#[cfg(not(target_os = "macos"))]
fn insets(_: &slint::winit_030::winit::monitor::MonitorHandle) -> (f64, f64) {
    (0.0, EXTRA_INSET)
}

/// (right, bottom) insets of the NSScreen behind `m`, from its visible frame.
#[cfg(target_os = "macos")]
fn work_area_insets(m: &slint::winit_030::winit::monitor::MonitorHandle) -> Option<(f64, f64)> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;

    let mtm = MainThreadMarker::new()?;
    let (mp, ms, scale) = (m.position(), m.size(), m.scale_factor());
    let matches = |s: &NSScreen| {
        let f = s.frame();
        (f.origin.x * scale - f64::from(mp.x)).abs() < 2.0
            && (f.size.width * scale - f64::from(ms.width)).abs() < 2.0
            && (f.size.height * scale - f64::from(ms.height)).abs() < 2.0
    };
    let screen = NSScreen::screens(mtm)
        .iter()
        .find(|s| matches(s))
        .or_else(|| NSScreen::mainScreen(mtm))?;
    let (f, v) = (screen.frame(), screen.visibleFrame());
    let bottom = v.origin.y - f.origin.y;
    let right = (f.origin.x + f.size.width) - (v.origin.x + v.size.width);
    Some((right.max(0.0), bottom.max(0.0)))
}
