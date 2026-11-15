#![allow(dead_code)]
//! Software-rendered Slint platform for snapshot tests: one fixed-size window, a manual clock and a
//! queue-backed event loop proxy, so renders and animations are deterministic and nothing is shown on screen.
use image::RgbaImage;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{
    EventLoopProxy, Platform, WindowAdapter, WindowEvent, update_timers_and_animations,
};
use slint::{EventLoopError, LogicalSize, PlatformError, Rgb8Pixel, SharedPixelBuffer};
use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const LOGICAL: (f32, f32) = (1280.0, 800.0);
pub const SCALE: f32 = 2.0;
const FRAME: Duration = Duration::from_millis(16);
const WAIT: Duration = Duration::from_secs(3);

type Job = Box<dyn FnOnce() + Send>;
type Queue = Arc<Mutex<VecDeque<Job>>>;

struct Proxy(Queue);

impl EventLoopProxy for Proxy {
    fn quit_event_loop(&self) -> Result<(), EventLoopError> {
        Ok(())
    }

    fn invoke_from_event_loop(&self, event: Job) -> Result<(), EventLoopError> {
        self.0
            .lock()
            .map_err(|_| EventLoopError::EventLoopTerminated)?
            .push_back(event);
        Ok(())
    }
}

#[derive(Clone)]
pub struct SwPlatform {
    window: Rc<MinimalSoftwareWindow>,
    clock: Rc<Cell<Duration>>,
    queue: Queue,
}

impl Platform for SwPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        self.clock.get()
    }

    fn new_event_loop_proxy(&self) -> Option<Box<dyn EventLoopProxy>> {
        Some(Box::new(Proxy(self.queue.clone())))
    }
}

impl SwPlatform {
    /// Installs the platform for this process; call once, before creating any component.
    pub fn install() -> SwPlatform {
        let p = SwPlatform {
            window: MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer),
            clock: Rc::default(),
            queue: Arc::default(),
        };
        p.window.dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: SCALE,
        });
        p.resize();
        slint::platform::set_platform(Box::new(p.clone())).expect("platform already set");
        p
    }

    /// Re-applies the fixed window size (showing a component may replace it with its preferred size).
    pub fn resize(&self) {
        self.window.set_size(LogicalSize::new(LOGICAL.0, LOGICAL.1));
    }

    fn run_jobs(&self) {
        loop {
            let job = self.queue.lock().ok().and_then(|mut q| q.pop_front());
            match job {
                Some(job) => job(),
                None => break,
            }
        }
    }

    /// Runs queued event-loop closures, advancing the clock one frame per round.
    pub fn pump(&self, rounds: u32) {
        for _ in 0..rounds {
            self.run_jobs();
            self.clock.set(self.clock.get() + FRAME);
            update_timers_and_animations();
            self.run_jobs();
        }
    }

    /// Pumps until `cond` holds; panics after 3 s of wall-clock time (worker threads need real time).
    pub fn pump_until(&self, what: &str, cond: impl Fn() -> bool) {
        let start = Instant::now();
        while !cond() {
            assert!(start.elapsed() < WAIT, "timed out waiting for: {what}");
            self.pump(1);
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    pub fn render(&self) -> RgbaImage {
        let size = self.window.size();
        let (w, h) = (size.width, size.height);
        let mut buf = SharedPixelBuffer::<Rgb8Pixel>::new(w, h);
        self.window.request_redraw();
        self.window.draw_if_needed(|r| {
            r.render(buf.make_mut_slice(), w as usize);
        });
        let px = buf.as_slice();
        RgbaImage::from_fn(w, h, |x, y| {
            let p = px[(y * w + x) as usize];
            image::Rgba([p.r, p.g, p.b, 255])
        })
    }
}
