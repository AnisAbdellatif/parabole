//! Runs UI-initiated database writes off the UI thread, in submission order.
//! The worker's refresh holds the write lock for seconds; a click must not wait for it.
use crate::db::Db;
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::mpsc::{Sender, channel};

pub type Job = Box<dyn FnOnce(&Db) + Send>;

pub struct Writer {
    tx: Sender<Job>,
}

impl Writer {
    /// Opens its own connection; the thread ends when the `Writer` is dropped.
    pub fn spawn(db_path: &Path) -> Result<Writer> {
        let db = Db::open(db_path).context("open writer database")?;
        let (tx, rx) = channel::<Job>();
        std::thread::Builder::new()
            .name("ui-writer".into())
            .spawn(move || {
                for job in rx {
                    job(&db);
                }
            })
            .context("spawn ui-writer")?;
        Ok(Writer { tx })
    }

    pub fn run(&self, job: impl FnOnce(&Db) + Send + 'static) {
        // send only fails once the thread is gone (app shutting down)
        let _ = self.tx.send(Box::new(job));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jobs_run_in_submission_order() {
        let path = std::env::temp_dir().join(format!("writer-test-{}.db", std::process::id()));
        let w = Writer::spawn(&path).unwrap();
        for v in ["first", "second"] {
            w.run(move |db| db.set_setting(None, "k", v).unwrap());
        }
        let (ack, done) = channel();
        w.run(move |db| ack.send(db.get_setting(None, "k").unwrap()).unwrap());
        assert_eq!(done.recv().unwrap().as_deref(), Some("second"));
        drop(w);
        for ext in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{ext}", path.display()));
        }
    }
}
