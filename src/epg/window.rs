#[derive(Clone, Copy, Debug)]
pub struct Window {
    pub from: i64,
    pub to: i64,
}

impl Window {
    /// 2 hours back to 3 days ahead of `now`.
    pub fn around(now: i64) -> Self {
        Self {
            from: now - 2 * 3600,
            to: now + 3 * 86400,
        }
    }
}
