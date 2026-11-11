use super::*;
use std::time::Instant;

#[test]
fn import_200k_streams() {
    let (mut db, p, s) = setup();
    let cats: Vec<String> = (0..200).map(|i| format!("c{i}")).collect();
    let mut data = ParsedPlaylist {
        categories: cats.iter().map(|c| cat(c)).collect(),
        ..Default::default()
    };
    data.streams = (0..200_000)
        .map(|i| {
            stream(
                &i.to_string(),
                &cats[i % 200],
                &format!("Channel number {i}"),
            )
        })
        .collect();
    let t = Instant::now();
    db.replace_source_data(s, &data).unwrap();
    let elapsed = t.elapsed();
    eprintln!("200k import: {elapsed:?}");
    assert!(elapsed.as_secs() < 30, "{elapsed:?}");
    let rows = db
        .search(p, None, MediaKind::Live, "chann 1999", 50)
        .unwrap();
    assert!(!rows.is_empty());
    let c = db.categories(p, None, MediaKind::Live, false).unwrap();
    assert_eq!(c[0].count, 1000);
}
