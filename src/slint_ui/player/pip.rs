//! Real picture-in-picture: a small frameless always-on-top window at the screen's bottom-right.
//! `AppWindow` binds `always-on-top`/`no-frame` to `PlayerApi.pip`; this only moves and sizes it.
use super::super::{AppWindow, PlayerApi};
use slint::winit_030::WinitWindowAccessor;
use slint::{ComponentHandle, LogicalSize, PhysicalPosition, PhysicalSize};

const SIZE: LogicalSize = LogicalSize::new(480.0, 270.0);
const MARGIN_X: f32 = 24.0;
// shortcut: fixed bottom margin clears a typical macOS dock; read the monitor's work area to do it right
const MARGIN_Y: f32 = 80.0;

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
        let x = mp.x + ms.width as i32 - ((SIZE.width + MARGIN_X) * scale) as i32;
        let y = mp.y + ms.height as i32 - ((SIZE.height + MARGIN_Y) * scale) as i32;
        window.set_position(PhysicalPosition::new(x, y));
    }
}

pub fn restore(w: &AppWindow, g: &Geometry) {
    w.window().set_size(g.size);
    w.window().set_position(g.pos);
    w.window().set_maximized(g.maximized);
}
