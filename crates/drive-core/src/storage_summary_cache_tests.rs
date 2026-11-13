//! Issue #89: the `GetOverview` storage block is a cached derived summary.
//!
//! These unit tests count the filesystem work (`StorageSummaryStats`) so the
//! hot path can be asserted to skip the drafts-directory enumeration and the
//! per-quarantine-record `stat` calls, and cover the freshness/error semantics
//! the additive `StorageSummary` fields expose. The scale curve is `#[ignore]`
//! and runs explicitly on the remote builder.
use crate::Store;
use crate::state::{
    STORAGE_SUMMARY_TTL_SECONDS, StorageSummaryStats, set_storage_summary_cache_bypass_for_red,
};
use rusqlite::params;
use std::io::Write;

fn draft_row(store: &Store, node: &str, token: &str, dirty: bool) {
    store
        .db
        .execute(
            "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty)
             VALUES(?1,?2,'v1',1,1,?3)",
            params![node, token, dirty],
        )
        .unwrap();
}

fn write_draft_file(store: &Store, token: &str, bytes: &[u8]) {
    let dir = store.root.join("drafts");
    std::fs::create_dir_all(&dir).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(dir.join(token))
        .unwrap();
    file.write_all(bytes).unwrap();
}

fn node_row(store: &Store, id: &str, deleted: bool) {
    store
        .db
        .execute(
            "INSERT OR REPLACE INTO nodes(id,parent,name,remote_version,directory,deleted)
             VALUES(?1,NULL,?2,'v1',0,?3)",
            params![id, format!("name-{id}"), deleted],
        )
        .unwrap();
}

fn delta(before: StorageSummaryStats, after: StorageSummaryStats) -> StorageSummaryStats {
    StorageSummaryStats {
        hits: after.hits - before.hits,
        refreshes: after.refreshes - before.refreshes,
        stale_served: after.stale_served - before.stale_served,
        scan_errors: after.scan_errors - before.scan_errors,
        directory_enumerations: after.directory_enumerations - before.directory_enumerations,
        file_stats: after.file_stats - before.file_stats,
    }
}

fn fixture(drafts: usize) -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let tokens: Vec<String> = (0..drafts)
        .map(|_| uuid::Uuid::new_v4().to_string())
        .collect();
    {
        let tx = store.db.unchecked_transaction().unwrap();
        for (index, token) in tokens.iter().enumerate() {
            let node = format!("n{index:07}");
            tx.execute(
                "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES(?1,NULL,?2,'v1',0)",
                params![node, format!("f{index}")],
            )
            .unwrap();
            tx.execute(
                "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty)
                 VALUES(?1,?2,'v1',1,1,0)",
                params![node, token],
            )
            .unwrap();
        }
        tx.commit().unwrap();
    }
    let drafts_dir = dir.path().join("drafts");
    std::fs::create_dir_all(&drafts_dir).unwrap();
    for token in &tokens {
        std::fs::write(drafts_dir.join(token), b"tiny body").unwrap();
    }
    (dir, store)
}

/// AC: the same storage generation served twice must not enumerate `drafts/`
/// or stat quarantine records again.
#[test]
fn repeat_overview_does_not_rescan_the_drafts_directory() {
    let (_dir, store) = fixture(50);
    let now = 1_000_000u64;
    let first = store.storage_summary_at(now).unwrap();
    assert_eq!(first.drafts_bytes, 50 * 9);
    assert_eq!(first.collected_at, now);
    assert!(!first.stale && first.error.is_none());

    let before = store.storage_summary_stats();
    let second = store.storage_summary_at(now).unwrap();
    let after = store.storage_summary_stats();
    let delta = delta(before, after);
    assert_eq!(
        delta.directory_enumerations, 0,
        "#89 warm overview enumerated the drafts/backup directories again: {delta:?}"
    );
    assert_eq!(
        delta.file_stats, 0,
        "#89 warm overview re-stat'ed draft/quarantine files: {delta:?}"
    );
    assert_eq!(delta.hits, 1, "#89 second read must be a cache hit: {delta:?}");
    assert_eq!(delta.refreshes, 0);
    assert_eq!(second, first);
}

/// Quarantine records are parsed and stat'ed on a refresh; a warm read must not
/// repeat those syscalls either.
#[test]
fn repeat_overview_does_not_stat_each_quarantine_record() {
    let (_dir, store) = fixture(2);
    let quarantine_dir = store.root.join("drafts-quarantine");
    std::fs::create_dir_all(&quarantine_dir).unwrap();
    let mut records = Vec::new();
    for index in 0..5 {
        let token = uuid::Uuid::new_v4().to_string();
        std::fs::write(quarantine_dir.join(&token), vec![b'q'; 7 + index]).unwrap();
        records.push(serde_json::json!({
            "token": token,
            "node": format!("n{index}"),
            "bytes": 7 + index,
            "quarantined_at": 1,
        }));
    }
    store
        .db
        .execute(
            "INSERT OR REPLACE INTO metadata(key,value) VALUES('drafts_quarantined',?1)",
            [serde_json::to_string(&records).unwrap()],
        )
        .unwrap();
    let now = 2_000_000u64;
    let first = store.storage_summary_at(now).unwrap();
    assert_eq!(first.quarantine_bytes, 7 + 8 + 9 + 10 + 11);

    let before = store.storage_summary_stats();
    let _ = store.storage_summary_at(now).unwrap();
    let delta = delta(before, store.storage_summary_stats());
    assert_eq!(
        delta.file_stats, 0,
        "#89 warm overview stat'ed quarantine records again: {delta:?}"
    );
    assert_eq!(delta.hits, 1);
}

/// A size change without a namespace commit must appear within the declared TTL
/// window; inside the window the cached block is served.
#[test]
fn draft_size_change_without_namespace_commit_updates_within_the_ttl() {
    let (_dir, store) = fixture(1);
    let token: String = store
        .db
        .query_row("SELECT token FROM drafts LIMIT 1", [], |row| row.get(0))
        .unwrap();
    let now = 3_000_000u64;
    let first = store.storage_summary_at(now).unwrap();
    assert_eq!(first.drafts_bytes, 9);

    // Same inode grows without any store event: tests the TTL calibration.
    write_draft_file(&store, &token, b"a much larger body");
    let inside_window = store.storage_summary_at(now).unwrap();
    assert_eq!(
        inside_window.drafts_bytes, 9,
        "inside the TTL window the published summary is served"
    );
    let after_window =
        store.storage_summary_at(now + STORAGE_SUMMARY_TTL_SECONDS + 1).unwrap();
    assert_eq!(
        after_window.drafts_bytes, 18,
        "after the TTL the size change is re-collected"
    );
    assert_eq!(after_window.collected_at, now + STORAGE_SUMMARY_TTL_SECONDS + 1);
}

/// An explicit draft event bumps the shared generation, so the next overview
/// refreshes immediately instead of waiting for the TTL.
#[test]
fn draft_events_bump_the_generation_and_refresh_immediately() {
    let (_dir, mut store) = fixture(1);
    node_row(&store, "n-live", false);
    let now = 4_000_000u64;
    let first = store.storage_summary_at(now).unwrap();
    let draft = store.begin_draft("n-live", "v1", None).unwrap();
    store.draft_write(&draft, 0, b"0123456789", false).unwrap();
    let after_write = store.storage_summary_at(now).unwrap();
    assert_eq!(
        after_write.drafts_bytes,
        first.drafts_bytes + 10,
        "a draft write must invalidate the summary without waiting for the TTL"
    );
    assert_eq!(after_write.drafts_bytes, first.drafts_bytes + 10);
}

/// The writer-shared generation invalidates a pooled read-only handle's summary
/// even when no commit happened (so `PRAGMA data_version` is unchanged).
#[test]
fn storage_generation_is_shared_with_read_only_overview_handles() {
    let (_dir, store) = fixture(1);
    node_row(&store, "n-live", false);
    let shared = crate::SharedStore::new(store);
    let now = 5_000_000u64;
    let draft = shared
        .lock()
        .unwrap()
        .begin_draft("n-live", "v1", None)
        .unwrap();
    // First write durably marks the dirty epoch: it commits, so the reader's
    // data_version moves and its cache invalidates through SQLite.
    shared
        .lock()
        .unwrap()
        .draft_write(&draft, 0, b"0123456789", false)
        .unwrap();
    let reader = shared.read_only().unwrap();
    let _ = reader.storage_summary_at(now).unwrap();
    let before = reader.storage_summary_stats();
    let _ = reader.storage_summary_at(now).unwrap();
    assert_eq!(delta(before, reader.storage_summary_stats()).hits, 1);

    let data_version = reader.data_version().unwrap();
    let size_before = reader.storage_summary_at(now).unwrap().drafts_bytes;
    // Second write in the same dirty epoch commits nothing (the epoch is already
    // durable) but bumps the shared storage generation: only the Arc hand-off
    // can make the read-only handle notice it.
    shared
        .lock()
        .unwrap()
        .draft_write(&draft, 10, b"abcdefghij", false)
        .unwrap();
    assert_eq!(
        reader.data_version().unwrap(),
        data_version,
        "second write must not commit; data_version is unchanged"
    );
    let before = reader.storage_summary_stats();
    let after = reader.storage_summary_at(now).unwrap();
    let delta = delta(before, reader.storage_summary_stats());
    assert_eq!(
        delta.refreshes, 1,
        "read-only handle must observe the writer's storage generation: {delta:?}"
    );
    assert_eq!(delta.hits, 0);
    assert_eq!(
        after.drafts_bytes,
        size_before + 10,
        "the refreshed summary includes the new bytes"
    );
}

/// AC: a failed scan must be explainable, never reported as zero statistics.
#[test]
fn failed_scan_returns_stale_error_instead_of_zero() {
    let (_dir, store) = fixture(3);
    let now = 6_000_000u64;
    let good = store.storage_summary_at(now).unwrap();
    assert_eq!(good.drafts_bytes, 27);

    let drafts_dir = store.root.join("drafts");
    let permissions = std::fs::metadata(&drafts_dir).unwrap().permissions();
    let mut blocked = permissions.clone();
    std::os::unix::fs::PermissionsExt::set_mode(&mut blocked, 0o000);
    std::fs::set_permissions(&drafts_dir, blocked).unwrap();
    let scanned = store.storage_summary_at(now + STORAGE_SUMMARY_TTL_SECONDS + 1);
    std::fs::set_permissions(&drafts_dir, permissions).unwrap();

    let stale = scanned.expect("#89 a failed scan must degrade, not fail the overview");
    assert!(stale.stale, "degraded summary must be marked stale");
    let error = stale.error.as_deref().unwrap_or_default();
    assert!(!error.is_empty(), "degraded summary must carry the scan error");
    assert_eq!(
        stale.drafts_bytes, good.drafts_bytes,
        "a failed refresh serves the last good numbers, not zero"
    );

    // A fresh handle with an unreadable drafts directory has no previous good
    // block: it returns an explicitly error-marked block instead of zeros.
    let (_dir2, store2) = fixture(1);
    let empty_drafts = store2.root.join("drafts");
    let permissions = std::fs::metadata(&empty_drafts).unwrap().permissions();
    let mut blocked = permissions.clone();
    std::os::unix::fs::PermissionsExt::set_mode(&mut blocked, 0o000);
    std::fs::set_permissions(&empty_drafts, blocked).unwrap();
    let degraded = store2.storage_summary_at(now);
    std::fs::set_permissions(&empty_drafts, permissions).unwrap();
    let degraded = degraded.expect("first-scan failure must still answer the overview");
    assert!(degraded.stale);
    assert!(degraded.error.is_some());
}

/// Orphan quarantine/restore changes the summary through the event path.
#[test]
fn quarantine_and_restore_update_the_summary() {
    let (_dir, mut store) = fixture(0);
    node_row(&store, "n-orphan", true);
    let token = uuid::Uuid::new_v4().to_string();
    draft_row(&store, "n-orphan", &token, true);
    write_draft_file(&store, &token, b"orphan body");
    // Past the orphan age window.
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(store.root.join("drafts").join(&token))
        .unwrap();
    file.set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1))
        .unwrap();
    drop(file);

    let now = 7_000_000u64;
    let before = store.storage_summary_at(now).unwrap();
    assert_eq!(before.drafts_bytes, 11);
    assert_eq!(before.drafts_reclaimable_bytes, 11);

    let quarantined = store.collect_orphan_drafts(now).unwrap();
    assert_eq!(quarantined.quarantined.len(), 1);
    let after = store.storage_summary_at(now).unwrap();
    assert_eq!(after.drafts_bytes, 0);
    assert_eq!(after.quarantine_bytes, 11);

    assert_eq!(store.restore_quarantined_drafts().unwrap(), 1);
    let restored = store.storage_summary_at(now).unwrap();
    assert_eq!(restored.drafts_bytes, 11);
    assert_eq!(restored.quarantine_bytes, 0);
}

/// AC: the hot path must not grow with the number of drafts. The deterministic
/// assertion is syscall counting; timings are printed for the report.
#[test]
#[ignore = "scaling evidence; run explicitly on the remote builder"]
fn storage_summary_scale_curve_and_hot_path() {
    for drafts in [0usize, 100, 1_000, 10_000, 50_000] {
        let (_dir, store) = fixture(drafts);
        let now = 8_000_000u64;
        let cold_started = std::time::Instant::now();
        let cold = store.storage_summary_at(now).unwrap();
        let cold_us = cold_started.elapsed().as_micros();
        let before = store.storage_summary_stats();
        let mut samples = Vec::with_capacity(500);
        for _ in 0..500 {
            let started = std::time::Instant::now();
            let _ = store.storage_summary_at(now).unwrap();
            samples.push(started.elapsed());
        }
        let delta = delta(before, store.storage_summary_stats());
        samples.sort_unstable();
        let pct = |p: f64| samples[((samples.len() - 1) as f64 * p) as usize].as_secs_f64() * 1e6;
        eprintln!(
            "STORAGE_CURVE drafts={drafts} cold_us={cold_us} warm_p50_us={:.2} warm_p95_us={:.2} \
             warm_p99_us={:.2} warm_max_us={:.2} warm_dir_enumerations={} warm_file_stats={} \
             hits={} drafts_bytes={}",
            pct(0.50),
            pct(0.95),
            pct(0.99),
            samples.last().unwrap().as_secs_f64() * 1e6,
            delta.directory_enumerations,
            delta.file_stats,
            delta.hits,
            cold.drafts_bytes,
        );
        assert_eq!(
            delta.directory_enumerations, 0,
            "drafts={drafts}: warm calls enumerated directories"
        );
        assert_eq!(delta.file_stats, 0, "drafts={drafts}: warm calls stat'ed files");
    }
}

/// Reproducible red, same tree: with the cache bypassed the pre-fix behavior
/// (one draft/quarantine/backup scan per read) is restored, so the hot-path
/// assertions above must fail; with the cache enabled they pass.
#[test]
fn cache_bypass_reproduces_the_red_scan_cost() {
    let (_dir, store) = fixture(20);
    let now = 9_000_000u64;
    set_storage_summary_cache_bypass_for_red(true);
    let before = store.storage_summary_stats();
    let _ = store.storage_summary_at(now).unwrap();
    let _ = store.storage_summary_at(now).unwrap();
    let red = delta(before, store.storage_summary_stats());
    set_storage_summary_cache_bypass_for_red(false);
    assert!(
        red.directory_enumerations >= 2 && red.file_stats >= 40,
        "red-89: uncached reads rescan drafts/backups every time: {red:?}"
    );
    assert_eq!(red.hits, 0);

    // Warm the fixed path once (the bypassed reads published nothing), then the
    // next two reads in the same generation/TTL window must not scan at all.
    let _ = store.storage_summary_at(now).unwrap();
    let before = store.storage_summary_stats();
    let _ = store.storage_summary_at(now).unwrap();
    let _ = store.storage_summary_at(now).unwrap();
    let fixed = delta(before, store.storage_summary_stats());
    assert_eq!(
        fixed.directory_enumerations, 0,
        "fixed path rescans only on generation/TTL change: {fixed:?}"
    );
    assert_eq!(fixed.hits, 2, "both warm reads are cache hits: {fixed:?}");
}
