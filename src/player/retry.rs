use std::time::Duration;

pub const MAX_RETRIES: u32 = 3;

/// Delay before retry number `attempt` (0-based) of a dropped live stream: 1s, 2s, 4s, then give up.
pub fn backoff(attempt: u32) -> Option<Duration> {
    (attempt < MAX_RETRIES).then(|| Duration::from_secs(1 << attempt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_retries_with_doubling_delay() {
        let secs: Vec<_> = (0..5).map(|a| backoff(a).map(|d| d.as_secs())).collect();
        assert_eq!(secs, [Some(1), Some(2), Some(4), None, None]);
    }
}
