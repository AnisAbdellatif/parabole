use super::*;
use std::time::Duration;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("iptv-imgcache-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn entry(age_secs: u64, len: u64, name: &str) -> Entry {
    Entry {
        modified: SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000 - age_secs),
        len,
        path: PathBuf::from(name),
    }
}

#[test]
fn file_name_is_stable_hex_and_distinct() {
    let a = file_name("http://a/logo.png");
    assert_eq!(a, file_name("http://a/logo.png"));
    assert_eq!(a.len(), 16);
    assert!(a.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_ne!(a, file_name("http://a/logo.jpg"));
    assert_eq!(file_name(""), "cbf29ce484222325");
}

#[test]
fn file_name_has_no_path_separators_for_hostile_urls() {
    let name = file_name("http://x/../../etc/passwd");
    assert!(name.bytes().all(|b| b.is_ascii_hexdigit()));
}

#[test]
fn plan_prune_drops_oldest_until_under_cap() {
    let entries = vec![
        entry(10, 40, "new"),
        entry(30, 40, "old"),
        entry(20, 40, "mid"),
    ];
    assert_eq!(plan_prune(entries, 80), [PathBuf::from("old")]);
    let entries = vec![entry(10, 40, "new"), entry(30, 40, "old")];
    assert!(plan_prune(entries, 80).is_empty());
    let entries = vec![entry(10, 40, "new"), entry(30, 40, "old")];
    assert_eq!(plan_prune(entries, 0).len(), 2);
}

#[test]
fn write_atomic_round_trips_and_leaves_no_tmp() {
    let dir = scratch("write");
    let path = dir.join(file_name("http://a/x.png"));
    write_atomic(&path, b"png").unwrap();
    assert_eq!(read_cached(&path).as_deref(), Some(&b"png"[..]));
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn read_cached_deletes_oversized_files() {
    let dir = scratch("big");
    let path = dir.join("big");
    fs::write(&path, vec![0u8; MAX_IMAGE_BYTES as usize + 1]).unwrap();
    assert!(read_cached(&path).is_none());
    assert!(!path.exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn prune_removes_oldest_files_from_disk() {
    let dir = scratch("prune");
    for (name, age) in [("old", 300), ("mid", 200), ("new", 100)] {
        let p = dir.join(name);
        fs::write(&p, [0u8; 10]).unwrap();
        let file = fs::File::options().write(true).open(&p).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(age))
            .unwrap();
    }
    assert_eq!(prune(&dir, 20).unwrap(), 1);
    assert!(!dir.join("old").exists() && dir.join("mid").exists() && dir.join("new").exists());
    fs::remove_dir_all(&dir).unwrap();
}
