use super::*;

fn m(status: MatchStatus, kickoff_utc: i64) -> Match {
    Match {
        key: "k".into(),
        competition: "c".into(),
        home: "h".into(),
        away: "a".into(),
        kickoff_utc,
        status,
        broadcasts: vec![],
    }
}

#[test]
fn tick_only_when_something_is_happening() {
    let now = 100_000;
    assert!(!wants_tick(&[], now));
    assert!(!wants_tick(&[m(MatchStatus::Upcoming, now + 3600)], now));
    assert!(wants_tick(&[m(MatchStatus::Upcoming, now + 30)], now));
    assert!(wants_tick(&[m(MatchStatus::Upcoming, now - 3600)], now));
    assert!(!wants_tick(
        &[m(MatchStatus::Upcoming, now - 4 * 3600)],
        now
    ));
    assert!(!wants_tick(&[m(MatchStatus::Finished, now - 60)], now));
    let live = MatchStatus::Live {
        clock: "1'".into(),
        home: "0".into(),
        away: "0".into(),
    };
    assert!(wants_tick(&[m(live, now - 9999)], now));
}
