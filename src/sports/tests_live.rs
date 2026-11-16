//! Live endpoint checks. Run: cargo test sports_live -- --ignored
//! A failure here means a provider changed or blocked us; see docs/football-tab.md §8.
use super::{catalog, client, livescore, s365, sportsdb};

fn today() -> chrono::NaiveDate {
    chrono::Utc::now().date_naive()
}

#[test]
#[ignore]
fn sports_live_365scores_day() {
    (catalog::football::SPORT.days[0].fetch)(&client().unwrap(), today()).unwrap();
}

#[test]
#[ignore]
fn sports_live_365scores_tv() {
    let client = client().unwrap();
    let matches = (catalog::football::SPORT.days[0].fetch)(&client, today()).unwrap();
    let Some(m) = matches.first() else {
        eprintln!("no 365scores matches today; skipping tv check");
        return;
    };
    s365::fetch_tv(&client, m, "DE").unwrap();
}

#[test]
#[ignore]
fn sports_live_livescore_day() {
    livescore::fetch_day(&client().unwrap(), today()).unwrap();
}

#[test]
#[ignore]
fn sports_live_basketball_day() {
    (catalog::basketball::SPORT.days[0].fetch)(&client().unwrap(), today()).unwrap();
}

#[test]
#[ignore]
fn sports_live_tennis_day() {
    (catalog::tennis::SPORT.days[0].fetch)(&client().unwrap(), today()).unwrap();
}

#[test]
#[ignore]
fn sports_live_f1_day_and_tv() {
    let client = client().unwrap();
    let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 22).unwrap();
    let matches = (catalog::f1::SPORT.days[0].fetch)(&client, date).unwrap();
    let m = matches.first().expect("an F1 race on 2026-11-22");
    sportsdb::fetch_tv(&client, m).unwrap();
}
