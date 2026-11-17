//! Smart tab "QUALITY SCAN" sidebar section: progress header and scanner actions.
use crate::slint_ui::SideRow;
use crate::slint_ui::state::SlintApp;
use crate::worker::ScanStatus;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

fn thousands(n: i64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn header(status: Option<&ScanStatus>, paused: bool) -> String {
    let tail = match status {
        Some(ScanStatus::Unavailable) => "ffprobe not found".to_string(),
        _ if paused => "paused".to_string(),
        Some(ScanStatus::Idle { .. }) => "done".to_string(),
        Some(ScanStatus::Running { done, total }) => {
            format!("{} / {}", thousands(*done), thousands(*total))
        }
        None => return "QUALITY SCAN".into(),
    };
    format!("QUALITY SCAN · {tail}")
}

fn row(id: &str, kind: i32, label: &str) -> SideRow {
    SideRow {
        id: id.into(),
        kind,
        label: label.into(),
        count: -1,
        icon: 0,
        pinned: false,
    }
}

impl SlintApp {
    pub(super) fn scan_rows(&self) -> Vec<SideRow> {
        let paused = self.worker.scan.paused.load(Ordering::Relaxed);
        vec![
            row("", 0, &header(self.scan_status.as_ref(), paused)),
            row("scan:choose", 1, "Choose categories…"),
            row(
                "scan:pause",
                1,
                if paused { "Resume scan" } else { "Pause scan" },
            ),
            row("scan:rescan", 1, "Rescan all"),
        ]
    }
}

/// Handles a `scan:*` sidebar id; false for any other id.
pub fn action(a: &mut SlintApp, id: &str) -> bool {
    match id {
        "scan:choose" => a.open_scan_dialog(),
        "scan:pause" => {
            a.worker.scan.paused.fetch_xor(true, Ordering::Relaxed);
            a.push_sidebar();
        }
        "scan:rescan" => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs() as i64);
            let profile = a.profile();
            a.write_with(
                move |db| db.set_scan_since(profile, now),
                |a, ok| {
                    if ok {
                        a.toast("Rescanning…");
                    }
                },
            );
        }
        _ => return false,
    }
    true
}

/// Stores the scanner status; reloads Smart data only when a scan just finished.
pub fn on_status(a: &mut SlintApp, s: ScanStatus) {
    let finished = matches!(a.scan_status, Some(ScanStatus::Running { .. }))
        && matches!(s, ScanStatus::Idle { .. });
    a.scan_status = Some(s);
    if !a.state.smart {
        return;
    }
    if finished {
        a.reload_categories();
    } else {
        a.push_sidebar();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_text() {
        assert_eq!(thousands(8300), "8,300");
        assert_eq!(thousands(1_234_567), "1,234,567");
        assert_eq!(thousands(12), "12");
        let run = ScanStatus::Running {
            done: 1240,
            total: 8300,
        };
        assert_eq!(header(Some(&run), false), "QUALITY SCAN · 1,240 / 8,300");
        assert_eq!(header(Some(&run), true), "QUALITY SCAN · paused");
        let idle = ScanStatus::Idle { done: 1, total: 1 };
        assert_eq!(header(Some(&idle), false), "QUALITY SCAN · done");
        let none = header(Some(&ScanStatus::Unavailable), true);
        assert_eq!(none, "QUALITY SCAN · ffprobe not found");
    }
}
