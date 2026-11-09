//! Issue #50 (R1): measure `ExclusionIndex::build` on the 210k-node fixture.
//!
//! Ignored by default because it copies and scans the shared 210k Store fixture
//! (~51 MiB). Run explicitly on the remote host:
//!
//! ```text
//! EXCLUDE_210K_FIXTURE=/home/caozijian/p032/fixtures/fix30-210k \
//!   cargo test -p drive-core --all-features --locked --test exclude_index_build_bench \
//!   -- --ignored --nocapture
//! ```
use drive_core::Store;
use drive_core::exclude::{ExcludeRules, ExclusionIndex};
use std::time::Instant;

fn copy_store(source: &std::path::Path, destination: &std::path::Path) {
    std::fs::create_dir_all(destination).expect("create fixture copy");
    for entry in std::fs::read_dir(source).expect("read fixture") {
        let entry = entry.expect("fixture entry");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("entry type").is_dir() {
            copy_store(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy fixture file");
        }
    }
}

#[test]
#[ignore = "R1 measurement: copies and scans the 210k-node Store fixture"]
fn exclusion_index_build_on_210k_fixture() {
    let source = std::env::var("EXCLUDE_210K_FIXTURE")
        .unwrap_or_else(|_| "/home/caozijian/p032/fixtures/fix30-210k".into());
    let source = std::path::PathBuf::from(source);
    assert!(
        source.join("state.sqlite").exists(),
        "fixture Store not found at {}",
        source.display()
    );
    let dir = tempfile::tempdir().expect("tempdir");
    let copy = dir.path().join("fixture");
    let started = Instant::now();
    copy_store(&source, &copy);
    eprintln!(
        "R1 fixture copy: {:.1} ms",
        started.elapsed().as_secs_f64() * 1000.0
    );
    let store = Store::open(&copy).expect("open fixture store");
    eprintln!(
        "R1 fixture: 210k live nodes (root + 105 dirs + files), root id = root"
    );

    let rules = ExcludeRules::compile(&[
        r"(^|/)\.tmp-",
        r"(^|/)node_modules/",
        r"\.iso$",
        r"^Pinned$",
    ])
    .expect("valid rules");
    for attempt in 1..=3 {
        let started = Instant::now();
        let index = ExclusionIndex::build(&store, "root", &rules).expect("build index");
        let elapsed = started.elapsed().as_secs_f64() * 1000.0;
        eprintln!(
            "R1 ExclusionIndex::build attempt {attempt}: {elapsed:.1} ms (entries={}, inodes={}, nodes={})",
            index.entries_len(),
            index.inodes_len(),
            index.nodes_len(),
        );
    }

    // A rule matching a hundred fixture directories exercises the DFS over the
    // excluded descendants (almost the whole namespace).
    let started = Instant::now();
    let matching = ExcludeRules::compile(&[r"^d0"]).expect("valid rule");
    let index = ExclusionIndex::build(&store, "root", &matching).expect("build matching index");
    eprintln!(
        "R1 matching-rule build: {:.1} ms (entries={}, inodes={}, nodes={})",
        started.elapsed().as_secs_f64() * 1000.0,
        index.entries_len(),
        index.inodes_len(),
        index.nodes_len(),
    );
}
