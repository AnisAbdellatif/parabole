//! Live endpoint checks. Run: cargo test sports_live -- --ignored
//! A failure here means a provider changed or blocked us; see docs/football-tab.md §8.
use super::{client, livescore, s365};

fn today() -> chrono::NaiveDate {
    chrono::Utc::now().date_naive()
}

#[test]
#[ignore]
fn sports_live_365scores_day() {
    s365::fetch_day(&client().unwrap(), today()).unwrap();
}

#[test]
#[ignore]
fn sports_live_365scores_tv() {
    let client = client().unwrap();
    let matches = s365::fetch_day(&client, today()).unwrap();
    let Some(m) = matches.first() else {
        eprintln!("no 365scores matches today; skipping tv check");
        return;
    };
    s365::fetch_tv(&client, s365::game_id(&m.key).unwrap(), "DE").unwrap();
}

#[test]
#[ignore]
fn sports_live_livescore_day() {
    livescore::fetch_day(&client().unwrap(), today()).unwrap();
}
