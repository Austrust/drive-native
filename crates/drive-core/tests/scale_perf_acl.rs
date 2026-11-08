//! Issue #26: ACL-denied subtree cost on lookup / getattr pages / READDIRPLUS.
//!
//! One ~294k-node fixture, `ACL_DENIED` access_denied rows, then:
//! * `posix_lookup_by_inode` / `Store::lookup` on a 2000-entry directory,
//! * a 256-path lookup batch,
//! * a 128-slot `directory_posix_page` (getattr projection page),
//! * a "legacy READDIRPLUS projection" loop: for every one of the 2000 entries
//!   run `posix_record_by_inode` followed by the exact `permissions::project`
//!   sequence (`enforce(Traverse)`, `filesystem_root`, `decide(List|Read)`,
//!   `mount_writable`, and for files `decide(WriteContent)`+`decide(Setattr)`),
//!   which is the SQL shape drive-fs pays per entry when any ACL denial exists.
//!
//! Run (release, one denied count per process so every phase sees a stable
//! generation):
//!
//! ```text
//! ACL_DENIED=1000 cargo test -p drive-core --release --all-features \
//!   --test scale_perf_acl -- --ignored --nocapture
//! ```
//!
//! Env: `ACL_DENIED` (default 1000), `ACL_LOOKUPS` (override iteration count),
//! `ACL_PROJECT` (override legacy projection entry count), `ACL_KEEP=1`.
//! The measured counts shrink automatically for the 5k/50k denied tiers so the
//! before run stays bounded; the per-operation numbers are like-for-like.

use std::path::Path;
use std::time::Instant;

use drive_core::filesystem::DirectorySlot;
use drive_core::write_permissions::{Action, Decision, decide, enforce, filesystem_root, mount_writable};
use drive_core::Store;
use rusqlite::{params, Connection};

// ---------------------------------------------------------------------------
// Fixture shape
// ---------------------------------------------------------------------------

const NODES_TARGET: usize = 294_033;
const BULK_FILES: usize = 2000;
const TOP_DIRS: usize = 100;
const SUB_DIRS_PER_TOP: usize = 45;
const SUB_DIRS: usize = TOP_DIRS * SUB_DIRS_PER_TOP;

const ROOT: usize = 0;
const BULK: usize = 1;
const BULK_FILE_START: usize = 2;
const TOP_START: usize = BULK_FILE_START + BULK_FILES;
const SUB_START: usize = TOP_START + TOP_DIRS;
const DIR_COUNT: usize = SUB_START + SUB_DIRS;
const FILES_TOTAL: usize = NODES_TARGET - DIR_COUNT;

fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn node_id(idx: usize) -> String {
    if idx == ROOT {
        return "root".to_string();
    }
    format!("{:016x}{:024x}", mix64(idx as u64), idx)
}

fn node_loc(idx: usize) -> (Option<String>, String, bool) {
    if idx == ROOT {
        return (None, "Drive".to_string(), true);
    }
    if idx == BULK {
        return (Some(node_id(ROOT)), "bulk".to_string(), true);
    }
    if idx < TOP_START {
        return (
            Some(node_id(BULK)),
            format!("b{:04}.dat", idx - BULK_FILE_START),
            false,
        );
    }
    if idx < SUB_START {
        return (
            Some(node_id(ROOT)),
            format!("d{:03}", idx - TOP_START),
            true,
        );
    }
    if idx < DIR_COUNT {
        let k = idx - SUB_START;
        return (
            Some(node_id(TOP_START + k / SUB_DIRS_PER_TOP)),
            format!("s{:02}", k % SUB_DIRS_PER_TOP),
            true,
        );
    }
    let ord = idx - DIR_COUNT;
    let sub = SUB_START + ord % SUB_DIRS;
    (Some(node_id(sub)), format!("f{:06}.dat", ord), false)
}

/// Distinct denied nodes, all outside the measured 2000-entry directory so the
/// lookup target stays visible while the recursive blocked closure still has to
/// be evaluated for every unrelated lookup (the issue #26 evidence shape).
fn denied_index(i: usize) -> usize {
    DIR_COUNT + i % FILES_TOTAL
}

fn bulk_file_name(i: usize) -> String {
    format!("b{:04}.dat", i % BULK_FILES)
}

// ---------------------------------------------------------------------------
// Fixture construction
// ---------------------------------------------------------------------------

struct Fixture {
    dir: tempfile::TempDir,
    store: Store,
    denied: usize,
    nodes: usize,
    build_ms: f64,
}

fn build_fixture(denied: usize) -> Result<Fixture, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let started = Instant::now();
    let store = Store::open(dir.path())?;
    let db = Connection::open(dir.path().join("state.sqlite"))?;
    db.execute_batch(
        "PRAGMA synchronous=OFF;
         PRAGMA journal_mode=WAL;
         PRAGMA cache_size=-65536;
         PRAGMA temp_store=MEMORY;
         BEGIN IMMEDIATE",
    )?;
    {
        let mut st = db.prepare(
            "INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob,\
             dirty_blob,base_version,pinned,deleted,conflict) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,0,?11)",
        )?;
        for idx in 0..NODES_TARGET {
            let (parent, name, directory) = node_loc(idx);
            st.execute(params![
                node_id(idx),
                parent,
                name,
                "1",
                directory,
                Option::<String>::None,
                Option::<String>::None,
                Option::<String>::None,
                Option::<String>::None,
                false,
                false,
            ])?;
        }
        let _ = st.finalize();
        // fs_inodes in insertion order: root is ino 1, bulk is ino 2 and the
        // bulk files are 3..=2002.
        db.execute_batch("INSERT INTO fs_inodes(node) SELECT id FROM nodes ORDER BY rowid;")?;
        db.execute_batch(
            "INSERT INTO node_metadata(node,size,mtime,mode) \
             SELECT id,4096,1,420 FROM nodes WHERE directory=0;",
        )?;
        let mut denied_st = db.prepare("INSERT INTO access_denied(node) VALUES (?1)")?;
        for i in 0..denied {
            denied_st.execute([node_id(denied_index(i))])?;
        }
        drop(denied_st);
        db.execute_batch("COMMIT")?;
    }
    let _ = db.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
    drop(db);
    Ok(Fixture {
        dir,
        store,
        denied,
        nodes: NODES_TARGET,
        build_ms: started.elapsed().as_secs_f64() * 1000.0,
    })
}

// ---------------------------------------------------------------------------
// Measurement helpers
// ---------------------------------------------------------------------------

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = ((sorted.len() as f64 - 1.0) * fraction).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

#[derive(Default)]
struct UsSamples {
    us: Vec<f64>,
}

impl UsSamples {
    fn record(&mut self, elapsed_us: f64) {
        self.us.push(elapsed_us);
    }

    fn summary(&self) -> (f64, f64, f64, usize) {
        let mut sorted = self.us.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        (
            percentile(&sorted, 0.5),
            percentile(&sorted, 0.95),
            sorted.last().copied().unwrap_or(0.0),
            sorted.len(),
        )
    }
}

/// The exact `drive_fs::permissions::project` sequence, expressed through the
/// public drive-core decision API (same statements, same order). Returns
/// `Some(mode)` when the projection is allowed to present attributes.
fn project_like(store: &Store, record: &mut drive_core::filesystem::PosixRecord) -> Option<()> {
    let Some(root) = filesystem_root(store).ok()? else {
        return Some(());
    };
    if enforce(store, &root, &record.id, Action::Traverse).is_err() {
        return None;
    }
    let root = filesystem_root(store).ok()??;
    let action = if record.directory {
        Action::List
    } else {
        Action::Read
    };
    if decide(store, &root, &record.id, action).ok()? == Decision::Deny {
        record.mode &= !0o444;
    }
    if !mount_writable(store, &root).ok()? {
        return Some(());
    }
    if record.directory {
        return Some(());
    }
    let mut all_denied = true;
    for action in [Action::WriteContent, Action::Setattr] {
        all_denied &= decide(store, &root, &record.id, action).ok()? == Decision::Deny;
    }
    if all_denied {
        record.mode &= !0o222;
    }
    Some(())
}

// ---------------------------------------------------------------------------
// The measurement
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn acl_denied_lookup_and_readdirplus_cost() -> Result<(), Box<dyn std::error::Error>> {
    let denied = env_usize("ACL_DENIED", 1000);
    let lookups = env_usize(
        "ACL_LOOKUPS",
        match denied {
            0..=2000 => 200,
            2001..=10000 => 40,
            _ => 20,
        },
    );
    let project_entries = env_usize(
        "ACL_PROJECT",
        if denied <= 5000 { BULK_FILES } else { 100 },
    );

    let fixture = build_fixture(denied)?;
    let store = &fixture.store;
    let bulk_id = node_id(BULK);
    let bulk_ino = (BULK + 1) as u64;
    let bulk_entries: Vec<(u64, String)> = (0..BULK_FILES)
        .map(|i| ((BULK_FILE_START + i + 1) as u64, bulk_file_name(i)))
        .collect();

    // Warm the statement cache and page cache once.
    for i in 0..2 {
        let _ = store.posix_lookup_by_inode(bulk_ino, &bulk_file_name(i));
    }

    // 1. FUSE lookup hot path: posix_lookup_by_inode on a 2000-entry directory.
    let mut lookup = UsSamples::default();
    for i in 0..lookups {
        let name = bulk_file_name(i);
        let started = Instant::now();
        let record = store.posix_lookup_by_inode(bulk_ino, &name)?;
        lookup.record(started.elapsed().as_secs_f64() * 1e6);
        assert_eq!(record.id, node_id(BULK_FILE_START + i % BULK_FILES));
    }
    let (lookup_p50, lookup_p95, lookup_max, _) = lookup.summary();

    // 2. Core namespace lookup (the non-inode path used by sync/reconcile).
    let mut core_lookup = UsSamples::default();
    for i in 0..lookups.min(100) {
        let name = bulk_file_name(i);
        let started = Instant::now();
        store.lookup(&bulk_id, &name)?;
        core_lookup.record(started.elapsed().as_secs_f64() * 1e6);
    }
    let (core_p50, core_p95, core_max, _) = core_lookup.summary();

    // 3. A 256-path lookup batch (issue evidence: 2.35-2.42s before the fix).
    let started = Instant::now();
    for i in 0..256 {
        let name = bulk_file_name(i);
        store.posix_lookup_by_inode(bulk_ino, &name)?;
    }
    let batch_256_ms = started.elapsed().as_secs_f64() * 1000.0;

    // 4. One 128-slot getattr projection page.
    let names: Vec<String> = (0..128).map(bulk_file_name).collect();
    let slots: Vec<DirectorySlot<'_>> = names
        .iter()
        .enumerate()
        .map(|(i, name)| DirectorySlot {
            ino: (BULK_FILE_START + i + 1) as u64,
            name,
            directory: false,
        })
        .collect();
    let mut page = UsSamples::default();
    for _ in 0..20 {
        let started = Instant::now();
        let records = store.directory_posix_page(bulk_ino, &slots)?;
        page.record(started.elapsed().as_secs_f64() * 1e6);
        assert_eq!(records.len(), slots.len());
    }
    let (page_p50, page_p95, page_max, _) = page.summary();

    // 4a. The ACL page variant (#26) must not materialize the access_blocked
    // closure, so its cost stays flat as denied rows grow.
    let mut acl_page = UsSamples::default();
    for _ in 0..20 {
        let started = Instant::now();
        let records = store.directory_posix_page_for_acl(bulk_ino, &slots)?;
        acl_page.record(started.elapsed().as_secs_f64() * 1e6);
        assert_eq!(records.len(), slots.len());
        assert!(records.iter().all(|record| record.is_some()));
    }
    let (acl_page_p50, acl_page_p95, acl_page_max, _) = acl_page.summary();
    println!(
        "#26 ACL PAGE {{p50_us:{:.1},p95_us:{:.1},max_us:{:.1}}}",
        acl_page_p50,
        acl_page_p95,
        acl_page_max,
    );

    // 5. Legacy READDIRPLUS projection: per-entry posix record + project().
    let mut projected = 0usize;
    let started = Instant::now();
    for i in 0..project_entries {
        let ino = bulk_entries[i % BULK_FILES].0;
        let mut record = store.posix_record_by_inode(ino)?;
        if project_like(store, &mut record).is_none() {
            // Denied entries are still sent as dirent-only replies; count the
            // cost the same way READDIRPLUS pays it.
        }
        projected += 1;
    }
    let legacy_project_ms = started.elapsed().as_secs_f64() * 1000.0;
    let legacy_project_per_entry_us = if projected == 0 {
        0.0
    } else {
        legacy_project_ms * 1000.0 / projected as f64
    };

    // 6. Batched ACL READDIRPLUS projection (#26): 128-entry pages, one
    // getattr page + one permission projection per page.
    let cache_before = store.permission_cache_stats();
    let mut batch_pages = 0usize;
    let mut batch_allowed = 0usize;
    let started = Instant::now();
    let mut index = 0usize;
    while index < project_entries {
        let end = (index + 128).min(project_entries).min(BULK_FILES);
        if end <= index {
            break;
        }
        let chunk = &bulk_entries[index..end];
        let page_slots: Vec<DirectorySlot<'_>> = chunk
            .iter()
            .map(|(ino, name)| DirectorySlot {
                ino: *ino,
                name,
                directory: false,
            })
            .collect();
        let page = store.directory_posix_page_for_acl(bulk_ino, &page_slots)?;
        let mut records: Vec<drive_core::filesystem::PosixRecord> =
            page.into_iter().flatten().collect();
        let allowed =
            drive_core::write_permissions::project_read_page(store, &bulk_id, &mut records)?;
        batch_allowed += allowed.iter().filter(|allowed| **allowed).count();
        batch_pages += 1;
        index = end;
    }
    let batch_project_ms = started.elapsed().as_secs_f64() * 1000.0;
    let cache_after = store.permission_cache_stats();

    // 6a. The page batch must agree with the per-entry project contract.
    let check_slots: Vec<DirectorySlot<'_>> = bulk_entries[..128]
        .iter()
        .map(|(ino, name)| DirectorySlot {
            ino: *ino,
            name,
            directory: false,
        })
        .collect();
    let mut check_records: Vec<drive_core::filesystem::PosixRecord> = store
        .directory_posix_page_for_acl(bulk_ino, &check_slots)?
        .into_iter()
        .flatten()
        .collect();
    let check_allowed =
        drive_core::write_permissions::project_read_page(store, &bulk_id, &mut check_records)?;
    let mut expected_allowed = 0usize;
    for (record, allowed) in check_records.iter().zip(check_allowed.iter()) {
        let mut legacy = record.clone();
        let legacy_allowed = project_like(store, &mut legacy).is_some();
        assert_eq!(
            *allowed, legacy_allowed,
            "batch/legacy allowed mismatch for {}",
            record.id
        );
        if *allowed {
            assert_eq!(record.mode, legacy.mode, "mode mismatch for {}", record.id);
            expected_allowed += 1;
        }
    }
    assert!(expected_allowed > 0, "no visible entries in the measured page");
    println!(
        "#26 ACL BATCH {{pages:{},allowed:{},entries:{},batch_project_ms:{:.3},batch_project_per_entry_us:{:.1},cache_hits:{},cache_misses:{}}}",
        batch_pages,
        batch_allowed,
        project_entries,
        batch_project_ms,
        if project_entries == 0 {
            0.0
        } else {
            batch_project_ms * 1000.0 / project_entries as f64
        },
        cache_after.hits.saturating_sub(cache_before.hits),
        cache_after.misses.saturating_sub(cache_before.misses),
    );

    println!(
        "#26 ACL {{nodes:{},dirs:{},files:{},denied:{},build_ms:{:.1},\
         lookup_n:{},lookup_p50_us:{:.1},lookup_p95_us:{:.1},lookup_max_us:{:.1},\
         core_lookup_p50_us:{:.1},core_lookup_p95_us:{:.1},core_lookup_max_us:{:.1},\
         batch256_ms:{:.3},page128_n:{},page128_p50_us:{:.1},page128_p95_us:{:.1},page128_max_us:{:.1},\
         legacy_project_n:{},legacy_project_ms:{:.1},legacy_project_per_entry_us:{:.1}}}",
        fixture.nodes,
        DIR_COUNT,
        FILES_TOTAL,
        fixture.denied,
        fixture.build_ms,
        lookups,
        lookup_p50,
        lookup_p95,
        lookup_max,
        core_p50,
        core_p95,
        core_max,
        batch_256_ms,
        page.us.len(),
        page_p50,
        page_p95,
        page_max,
        projected,
        legacy_project_ms,
        legacy_project_per_entry_us,
    );

    let keep = std::env::var("ACL_KEEP").as_deref() == Ok("1");
    if keep {
        let dir = fixture.dir.keep();
        println!("#26 ACL fixture kept at {}", dir.display());
    }
    Ok(())
}
