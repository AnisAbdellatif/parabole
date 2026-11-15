//! Image loader: disk-cached downloads decoded off the UI thread into one LRU per size class.
mod fx;

use crate::imgcache;
use image::RgbaImage;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};

const WORKERS: usize = 4;

pub struct Lru<V> {
    map: HashMap<String, (V, u64)>,
    tick: u64,
    cap: usize,
}

impl<V: Clone> Lru<V> {
    pub fn new(cap: usize) -> Self {
        Self {
            map: HashMap::new(),
            tick: 0,
            cap,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<V> {
        self.tick += 1;
        let tick = self.tick;
        let (v, t) = self.map.get_mut(key)?;
        *t = tick;
        Some(v.clone())
    }

    pub fn insert(&mut self, key: String, v: V) {
        self.tick += 1;
        self.map.insert(key, (v, self.tick));
        if self.map.len() > self.cap
            && let Some(old) = self
                .map
                .iter()
                .min_by_key(|(_, (_, t))| *t)
                .map(|(k, _)| k.clone())
        {
            self.map.remove(&old);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Size {
    /// 96px thumbnail for list rows.
    Logo,
    /// 360px wide poster for tiles and the series page.
    Poster,
    /// 480px wide, blurred and darkened, for the hero.
    Backdrop,
}

const SIZES: [Size; 3] = [Size::Logo, Size::Poster, Size::Backdrop];

impl Size {
    fn idx(self) -> usize {
        self as usize
    }

    fn cap(self) -> usize {
        match self {
            Size::Logo => 600,
            Size::Poster => 120,
            Size::Backdrop => 16,
        }
    }
}

pub struct Decoded {
    pub buf: SharedPixelBuffer<Rgba8Pixel>,
}

type Key = (Size, String);

pub struct Images {
    lru: RefCell<[Lru<Image>; 3]>,
    pending: RefCell<HashSet<Key>>,
    failed: RefCell<HashSet<Key>>,
    tx: Option<Sender<Key>>,
}

fn is_http(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn decode(bytes: &[u8], size: Size) -> Option<Decoded> {
    let src = image::load_from_memory(bytes).ok()?;
    let small: RgbaImage = match size {
        Size::Logo => src.thumbnail(96, 96),
        Size::Poster => src.thumbnail(360, 540),
        Size::Backdrop => src.thumbnail(480, 480),
    }
    .to_rgba8();
    let out = if size == Size::Backdrop {
        fx::backdrop(&small)
    } else {
        small
    };
    let buf = SharedPixelBuffer::clone_from_slice(out.as_raw(), out.width(), out.height());
    Some(Decoded { buf })
}

impl Images {
    /// `on_ready` runs on a worker thread: `Some(decoded)` on success, `None` on failure.
    pub fn new(
        on_ready: impl Fn(String, Size, Option<Decoded>) + Send + Sync + 'static,
    ) -> Rc<Self> {
        let tx = imgcache::default_dir()
            .zip(imgcache::http_client().ok())
            .map(|(dir, client)| {
                let (tx, rx) = channel::<Key>();
                let rx = Arc::new(Mutex::new(rx));
                let on_ready = Arc::new(on_ready);
                for _ in 0..WORKERS {
                    let (rx, client, dir, on_ready) =
                        (rx.clone(), client.clone(), dir.clone(), on_ready.clone());
                    std::thread::spawn(move || {
                        loop {
                            let job = match rx.lock() {
                                Ok(rx) => rx.recv(),
                                Err(_) => return,
                            };
                            let Ok((size, url)) = job else { return };
                            let dec = imgcache::get_or_fetch(&dir, &client, &url)
                                .ok()
                                .and_then(|b| decode(&b, size));
                            on_ready(url, size, dec);
                        }
                    });
                }
                tx
            });
        Rc::new(Self {
            lru: RefCell::new(SIZES.map(|s| Lru::new(s.cap()))),
            pending: Default::default(),
            failed: Default::default(),
            tx,
        })
    }

    fn cached(&self, url: &str, size: Size) -> Option<Image> {
        self.lru.borrow_mut()[size.idx()].get(url)
    }

    /// Cached image or None; on a miss, queues a fetch (once per URL and size).
    pub fn get(&self, url: &str, size: Size) -> Option<Image> {
        let tx = self.tx.as_ref()?;
        if !is_http(url) {
            return None;
        }
        if let Some(img) = self.cached(url, size) {
            return Some(img);
        }
        let key = (size, url.to_owned());
        if !self.failed.borrow().contains(&key) && self.pending.borrow_mut().insert(key.clone()) {
            let _ = tx.send(key);
        }
        None
    }

    pub fn insert(&self, url: String, size: Size, d: Decoded) {
        self.pending.borrow_mut().remove(&(size, url.clone()));
        self.lru.borrow_mut()[size.idx()].insert(url, Image::from_rgba8(d.buf));
    }

    pub fn fail(&self, url: String, size: Size) {
        self.pending.borrow_mut().remove(&(size, url.clone()));
        self.failed.borrow_mut().insert((size, url));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lru_evicts_least_recently_used() {
        let mut l = Lru::new(2);
        l.insert("a".into(), 1);
        l.insert("b".into(), 2);
        l.get("a");
        l.insert("c".into(), 3);
        assert!(l.get("b").is_none());
        assert!(l.get("a").is_some() && l.get("c").is_some());
    }

    #[test]
    fn cache_keys_separate_sizes() {
        let imgs = Images::new(|_, _, _| {});
        let d = || Decoded {
            buf: SharedPixelBuffer::new(2, 2),
        };
        imgs.insert("http://x/a".into(), Size::Poster, d());
        assert!(imgs.cached("http://x/a", Size::Poster).is_some());
        assert!(imgs.cached("http://x/a", Size::Logo).is_none());
        assert!(imgs.cached("http://x/a", Size::Backdrop).is_none());
    }
}
