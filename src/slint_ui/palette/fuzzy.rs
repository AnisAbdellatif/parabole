//! Subsequence scoring for ranking palette categories and actions.

/// Case-insensitive subsequence match. Higher is better; `None` when `query` isn't a subsequence.
/// Bonus for word starts and consecutive runs, small penalty for long text.
// shortcut: greedy after the first char (best over all first-char positions), not a full DP.
pub fn score(query: &str, text: &str) -> Option<i32> {
    let q: Vec<char> = query
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect();
    let t: Vec<char> = text.chars().collect();
    let Some(&first) = q.first() else {
        return Some(0);
    };
    let starts = (0..t.len()).filter(|&i| lower(t[i]) == Some(first));
    starts
        .filter_map(|from| greedy(&q, &t, from))
        .max()
        .map(|s| s - t.len() as i32 / 4)
}

fn lower(c: char) -> Option<char> {
    c.to_lowercase().next()
}

fn greedy(q: &[char], t: &[char], from: usize) -> Option<i32> {
    let (mut qi, mut run, mut score) = (0, 0, 0);
    for i in from..t.len() {
        if qi == q.len() || lower(t[i]) != Some(q[qi]) {
            run = 0;
            continue;
        }
        let word_start = i == 0 || !t[i - 1].is_alphanumeric();
        score += 1 + if word_start { 8 } else { 0 } + run * 3;
        run += 1;
        qi += 1;
    }
    (qi == q.len()).then_some(score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_matches_are_none() {
        assert_eq!(score("xyz", "Refresh all sources"), None);
        assert_eq!(score("sr", "Refresh"), None);
        assert_eq!(score("abc", ""), None);
    }

    #[test]
    fn empty_query_matches_everything() {
        assert!(score("", "Open settings").is_some());
    }

    #[test]
    fn case_insensitive() {
        assert!(score("OPEN", "open settings").is_some());
        assert!(score("open", "OPEN SETTINGS").is_some());
    }

    #[test]
    fn prefix_beats_scattered() {
        assert!(score("set", "Settings") > score("set", "Open sources then tabs"));
    }

    #[test]
    fn word_starts_beat_mid_word() {
        assert!(score("os", "Open sources") > score("os", "Cosmos"));
    }

    #[test]
    fn consecutive_beats_split() {
        assert!(score("pro", "Open profiles") > score("pro", "Player rotation order"));
    }
}
