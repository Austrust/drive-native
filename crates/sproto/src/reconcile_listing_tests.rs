//! Reproducible listing-cost tests for namespace reconciliation.
//!
//! `SCMD_LIST` has no by-name filter, so every path lookup enumerates its whole
//! parent directory. These tests pin the resulting RPC counts against an
//! in-process protocol fixture; no NAS is involved.

use crate::actions::{
    list::FileType,
    mutations::ReadBackIndex,
    namespace::{NamespaceTarget, RenameReconciliation},
    native::NativeVersion,
};
use crate::test_support;

const VIEW: u64 = 54;

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
