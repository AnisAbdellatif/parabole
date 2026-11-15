//! Logo loader: disk-cached downloads decoded off the UI thread into a small LRU.
use crate::imgcache;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};

const WORKERS: usize = 4;
const CAP: usize = 600;
const THUMB: u32 = 96;

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

pub struct Images {
    lru: RefCell<Lru<Image>>,
    pending: RefCell<HashSet<String>>,
    failed: RefCell<HashSet<String>>,
    tx: Option<Sender<String>>,
}

fn is_http(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn decode(bytes: &[u8]) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let img = image::load_from_memory(bytes)
        .ok()?
        .thumbnail(THUMB, THUMB)
        .to_rgba8();
    Some(SharedPixelBuffer::clone_from_slice(
        img.as_raw(),
        img.width(),
        img.height(),
    ))
}

impl Images {
    /// `on_ready` runs on a worker thread: `Some(buf)` on success, `None` on failure.
    pub fn new(
        on_ready: impl Fn(String, Option<SharedPixelBuffer<Rgba8Pixel>>) + Send + Sync + 'static,
    ) -> Rc<Self> {
        let tx = imgcache::default_dir()
            .zip(imgcache::http_client().ok())
            .map(|(dir, client)| {
                let (tx, rx) = channel::<String>();
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
                            let Ok(url) = job else { return };
                            let buf = imgcache::get_or_fetch(&dir, &client, &url)
                                .ok()
                                .and_then(|b| decode(&b));
                            on_ready(url, buf);
                        }
                    });
                }
                tx
            });
        Rc::new(Self {
            lru: RefCell::new(Lru::new(CAP)),
            pending: Default::default(),
            failed: Default::default(),
            tx,
        })
    }

    /// Cached image or None; on a miss, queues a fetch (once per URL).
    pub fn get(&self, url: &str) -> Option<Image> {
        let tx = self.tx.as_ref()?;
        if !is_http(url) {
            return None;
        }
        if let Some(img) = self.lru.borrow_mut().get(url) {
            return Some(img);
        }
        if !self.failed.borrow().contains(url) && self.pending.borrow_mut().insert(url.to_owned()) {
            let _ = tx.send(url.to_owned());
        }
        None
    }

    pub fn insert(&self, url: String, buf: SharedPixelBuffer<Rgba8Pixel>) {
        self.pending.borrow_mut().remove(&url);
        self.lru.borrow_mut().insert(url, Image::from_rgba8(buf));
    }

    pub fn fail(&self, url: String) {
        self.pending.borrow_mut().remove(&url);
        self.failed.borrow_mut().insert(url);
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
}
