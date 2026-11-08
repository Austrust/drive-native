//! Reproducible listing-cost tests for namespace reconciliation.
//!
//! `SCMD_LIST` has no by-name filter, so every path lookup enumerates its whole
//! parent directory. These tests pin the resulting RPC counts against an
//! in-process protocol fixture; no NAS is involved.

use crate::actions::{
    list::FileType,
    mutations::{ReadBackIndex, UploadIntent, UploadReconciliation},
    namespace::{NamespaceTarget, RemoveReconciliation, RenameReconciliation},
    native::{EventCursor, NativeVersion, Watch},
};
use crate::pstream::PObject;
use crate::test_support;

const VIEW: u64 = 54;
/// SHA-256 of `b"abc"`.
const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

fn target(path: &str) -> NamespaceTarget {
    NamespaceTarget {
        view_id: VIEW,
        path: path.into(),
        file_id: "19".into(),
        base: NativeVersion {
            sync_id: 7,
            max_id: 7,
        },
        file_type: FileType::File,
    }
}

/// Baseline: each `current_path` lookup is a fresh full parent enumeration.
/// A 10k-entry directory needs 10 pages per lookup, so four lookups cost 40.
#[tokio::test]
async fn repeated_lookups_enumerate_the_parent_once_per_lookup() {
    let fixture = test_support::start().await;
    fixture.set_files("/big", 10_000);
    let client = fixture.connect(2).await;
    fixture.reset_stats();

    for _ in 0..4 {
        assert!(
            client.current_path(VIEW, "/big/absent").await.unwrap().is_none()
        );
    }

    let stats = fixture.stats();
    assert_eq!(stats.list_calls, 4, "one enumeration per lookup");
    assert_eq!(stats.list_pages, 40, "10 pages per lookup at 1000/page");
}

/// Regression for issue #33: a same-parent rename reads source and destination
/// in each pass. Both lookups must share one enumeration, cutting the cost from
/// 4 enumerations / 40 pages to 2 enumerations / 20 pages.
#[tokio::test]
async fn same_parent_rename_shares_one_enumeration_per_pass() {
    let fixture = test_support::start().await;
    fixture.set_files("/big", 10_000);
    let client = fixture.connect(2).await;
    fixture.reset_stats();

    let result = client
        .reconcile_rename(&target("/big/old-name"), "/big/new-name")
        .await
        .unwrap();
    assert!(matches!(
        result,
        RenameReconciliation::Diverged {
            source: None,
            destination: None
        }
    ));

    let stats = fixture.stats();
    assert_eq!(
        stats.list_calls, 2,
        "before-pass and after-pass each enumerate /big once"
    );
    assert_eq!(stats.list_pages, 20, "10 pages for each of the two passes");
}

/// Different parents cannot share an enumeration: each pass still needs one
/// enumeration per parent, and the two passes stay independent.
#[tokio::test]
async fn different_parent_rename_keeps_one_enumeration_per_parent_per_pass() {
    let fixture = test_support::start().await;
    fixture.set_files("/dir-one", 10_000);
    fixture.set_files("/dir-two", 10_000);
    let client = fixture.connect(2).await;
    fixture.reset_stats();

    let result = client
        .reconcile_rename(&target("/dir-one/old-name"), "/dir-two/new-name")
        .await
        .unwrap();
    assert!(matches!(result, RenameReconciliation::Diverged { .. }));

    let stats = fixture.stats();
    assert_eq!(stats.list_calls, 4);
    assert_eq!(stats.list_pages, 40);
}

/// The read-back index itself never enumerates the same parent twice, including
/// for several lookups of different leaves.
#[tokio::test]
async fn read_back_index_enumerates_each_parent_once() {
    let fixture = test_support::start().await;
    fixture.set_files("/big", 10_000);
    let client = fixture.connect(2).await;
    fixture.reset_stats();

    let mut index = ReadBackIndex::default();
    for path in ["/big/a", "/big/b", "/big/c"] {
        assert!(
            index
                .current_path(&client, VIEW, path)
                .await
                .unwrap()
                .is_none()
        );
    }

    let stats = fixture.stats();
    assert_eq!(stats.list_calls, 1);
    assert_eq!(stats.list_pages, 10);
}

fn remove_target() -> NamespaceTarget {
    NamespaceTarget {
        view_id: VIEW,
        path: "/big/f00000".into(),
        file_id: "100".into(),
        base: NativeVersion {
            sync_id: 7,
            max_id: 7,
        },
        file_type: FileType::File,
    }
}

fn remove_watch() -> Watch {
    Watch {
        view_id: VIEW,
        node_id: 3,
        path: "/big".into(),
        cursor: EventCursor(7),
    }
}

fn pull_page(next: u64, events: Vec<PObject>) -> PObject {
    pmap! {
        "next_sync_id" => next,
        "event_list" => events,
    }
}

fn remove_event() -> PObject {
    pmap! {
        "path" => "/big/f00000",
        "type" => "remove",
        "sync_id" => 8u64,
        "file_id" => "100",
        "file" => pmap! {
            "file_type" => "file",
            "is_removed" => 1u64,
            "max_id" => 8u64,
        },
    }
}

fn upload_intent(mtime: Option<u32>) -> UploadIntent {
    UploadIntent {
        view_id: VIEW,
        path: "/big/f00000".into(),
        original_file_id: None,
        base: NativeVersion {
            sync_id: 7,
            max_id: 7,
        },
        size: 3,
        md4: test_support::md4_hex(b"abc"),
        sha256: ABC_SHA256.into(),
        mtime,
    }
}

/// Populate `/big` with 10k entries whose first row owns content `b"abc"`.
fn content_directory(fixture: &test_support::Fixture) {
    let mut nodes: Vec<PObject> = (0..10_000)
        .map(|index| test_support::file_node(index, "file"))
        .collect();
    nodes[0] = test_support::file_node_with_content(0, b"abc");
    fixture.set_directory("/big", nodes);
    fixture.set_download("100", b"abc".to_vec());
}

/// Regression for issue #33 condition 1: an empty advancing pull page proves
/// the watched subtree did not change, so remove reconciliation may reuse the
/// first enumeration instead of enumerating the parent twice (2/20 -> 1/10).
#[tokio::test]
async fn same_parent_remove_shares_one_enumeration_without_changes() {
    let fixture = test_support::start().await;
    fixture.set_files("/big", 10_000);
    fixture.set_pull_responses(vec![pull_page(7, Vec::new())]);
    let client = fixture.connect(2).await;
    fixture.reset_stats();

    let result = client
        .reconcile_remove(&remove_target(), &remove_watch())
        .await
        .unwrap();
    assert!(matches!(result, RemoveReconciliation::StillPresent(_)));

    let stats = fixture.stats();
    assert_eq!(stats.pull_calls, 1);
    assert_eq!(
        stats.list_calls, 1,
        "one enumeration shared by the before/after read-back"
    );
    assert_eq!(stats.list_pages, 10);
}

/// When the pull page carries events, the after observation must be fresh: the
/// target is still present afterwards, so the result is unchanged and both
/// lookups stay independent (2 enumerations / 20 pages).
#[tokio::test]
async fn remove_with_observed_events_keeps_a_fresh_observation() {
    let fixture = test_support::start().await;
    fixture.set_files("/big", 10_000);
    fixture.set_pull_responses(vec![pull_page(8, vec![remove_event()])]);
    let client = fixture.connect(2).await;
    fixture.reset_stats();

    let result = client
        .reconcile_remove(&remove_target(), &remove_watch())
        .await
        .unwrap();
    assert!(matches!(result, RemoveReconciliation::StillPresent(_)));

    let stats = fixture.stats();
    assert_eq!(stats.pull_calls, 1);
    assert_eq!(stats.list_calls, 2);
    assert_eq!(stats.list_pages, 20);
}

/// Regression for issue #33 condition 1: the upload read-back shares the
/// parent enumeration for implicit-mtime uploads, where the verified download
/// is already an authoritative observation (2/20 -> 1/10).
#[tokio::test]
async fn same_parent_upload_read_back_shares_one_enumeration() {
    let fixture = test_support::start().await;
    content_directory(&fixture);
    let client = fixture.connect(2).await;
    fixture.reset_stats();
    let staging = tempfile::tempdir().unwrap();

    let result = client
        .reconcile_upload(&upload_intent(None), staging.path())
        .await
        .unwrap();
    assert!(matches!(
        result,
        UploadReconciliation::DesiredStatePresent(_)
    ));

    let stats = fixture.stats();
    assert_eq!(stats.download_calls, 1);
    assert_eq!(
        stats.list_calls, 1,
        "the download covers the implicit-mtime read-back"
    );
    assert_eq!(stats.list_pages, 10);
}

/// Explicit-mtime uploads still need a fresh read-back so the mtime check runs
/// against a first-hand observation (2 enumerations / 20 pages).
#[tokio::test]
async fn upload_with_explicit_mtime_keeps_a_fresh_read_back() {
    let fixture = test_support::start().await;
    content_directory(&fixture);
    let client = fixture.connect(2).await;
    fixture.reset_stats();
    let staging = tempfile::tempdir().unwrap();

    let result = client
        .reconcile_upload(&upload_intent(Some(1_700_000_000)), staging.path())
        .await
        .unwrap();
    assert!(matches!(
        result,
        UploadReconciliation::DesiredStatePresent(_)
    ));

    let stats = fixture.stats();
    assert_eq!(stats.download_calls, 1);
    assert_eq!(stats.list_calls, 2);
    assert_eq!(stats.list_pages, 20);
}
