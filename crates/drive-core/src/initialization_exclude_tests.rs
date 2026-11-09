//! Issue #50: excluded subtrees never enter an initialization candidate.
use super::*;
use crate::exclude::ExcludeRules;
use crate::initialization_events::InitializationDirectoryEntry;

fn entries(nodes: &[Node]) -> Vec<InitializationDirectoryEntry> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| InitializationDirectoryEntry {
            node: node.clone(),
            native_node_id: 9_000 + index as u64,
        })
        .collect()
}

#[test]
fn filtered_directory_receipt_drops_excluded_children_before_import() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let rules = ExcludeRules::compile(&[r"(^|/)node_modules/", r"\.iso$"]).unwrap();
    let ticket = store.begin_initialization_directory(&run, "100").unwrap();
    let children = [
        node("101", Some("100"), "keep.txt", false, 7, "server"),
        node("102", Some("100"), "node_modules", true, 7, "server"),
        node("103", Some("100"), "image.iso", false, 7, "server"),
    ];
    assert!(
        store
            .store_initialization_native_directory_filtered(&ticket, 1, &entries(&children), &rules)
            .unwrap()
    );
    assert!(store.initialization_node(&run, "101").is_ok());
    assert!(store.initialization_node(&run, "102").is_err());
    assert!(store.initialization_node(&run, "103").is_err());
    let coverage = store.initialization_coverage(&run).unwrap();
    assert_eq!(coverage.len(), 1, "{coverage:?}");
    assert!(
        store
            .initialization_pending_directories(&run, 16)
            .unwrap()
            .is_empty(),
        "an excluded directory was scheduled for enumeration"
    );
    store.test_fence(&run, "native-v1:100").unwrap();
    store.complete_initialization(&run).unwrap();
    assert_eq!(store.node("101").unwrap().name, "keep.txt");
    assert!(store.node("102").is_err());
    assert!(store.node("103").is_err());
}

#[test]
fn filtered_receipt_keeps_unmatched_directory_and_filters_its_children() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let rules = ExcludeRules::compile(&[r"(^|/)node_modules/"]).unwrap();
    let root = store.begin_initialization_directory(&run, "100").unwrap();
    assert!(
        store
            .store_initialization_native_directory_filtered(
                &root,
                1,
                &entries(&[node("200", Some("100"), "Sub", true, 7, "server")]),
                &rules,
            )
            .unwrap()
    );
    // The child receipt is matched against the scope-relative path
    // `Sub/node_modules`, not against the bare name.
    let sub = store.begin_initialization_directory(&run, "200").unwrap();
    let children = [
        node("201", Some("200"), "C.txt", false, 7, "server"),
        node("202", Some("200"), "node_modules", true, 7, "server"),
    ];
    assert!(
        store
            .store_initialization_native_directory_filtered(&sub, 1, &entries(&children), &rules)
            .unwrap()
    );
    assert!(store.initialization_node(&run, "201").is_ok());
    assert!(store.initialization_node(&run, "202").is_err());
    store.test_fence(&run, "native-v1:100").unwrap();
    store.complete_initialization(&run).unwrap();
    assert!(store.node("200").unwrap().directory);
    assert_eq!(store.node("201").unwrap().name, "C.txt");
    assert!(store.node("202").is_err());
}

#[test]
fn prune_removes_a_candidate_subtree_built_under_older_rules() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(
        &mut store,
        &run,
        "100",
        &[
            node("101", Some("100"), "keep.txt", false, 7, "server"),
            node("200", Some("100"), "Sub", true, 7, "server"),
        ],
    );
    list(
        &mut store,
        &run,
        "200",
        &[
            node("201", Some("200"), "C.txt", false, 7, "server"),
            node("202", Some("200"), "node_modules", true, 7, "server"),
        ],
    );
    list(
        &mut store,
        &run,
        "202",
        &[node("203", Some("202"), "inner.js", false, 7, "server")],
    );
    let rules = ExcludeRules::compile(&[r"(^|/)node_modules$"]).unwrap();
    assert_eq!(store.prune_initialization_excluded(&run, &rules).unwrap(), 1);
    assert!(store.initialization_node(&run, "201").is_ok());
    assert!(store.initialization_node(&run, "202").is_err());
    assert!(store.initialization_node(&run, "203").is_err());
    let coverage = store.initialization_coverage(&run).unwrap();
    assert!(!coverage.iter().any(|entry| entry.directory == "202"));
    let pending = store.initialization_pending_directories(&run, 16).unwrap();
    assert!(!pending.iter().any(|entry| entry.id == "202"));
    // A second prune is a no-op.
    assert_eq!(store.prune_initialization_excluded(&run, &rules).unwrap(), 0);
    store.test_fence(&run, "native-v1:100").unwrap();
    store.complete_initialization(&run).unwrap();
    assert!(store.node("200").unwrap().directory);
    assert_eq!(store.node("201").unwrap().name, "C.txt");
    assert!(store.node("202").is_err());
    assert!(store.node("203").is_err());
}

#[test]
fn prune_excludes_the_whole_subtree_for_an_ancestor_rule() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(
        &mut store,
        &run,
        "100",
        &[node("200", Some("100"), "Pinned", true, 7, "server")],
    );
    list(
        &mut store,
        &run,
        "200",
        &[node("201", Some("200"), "deep.txt", false, 7, "server")],
    );
    let rules = ExcludeRules::compile(&[r"^Pinned$"]).unwrap();
    assert_eq!(store.prune_initialization_excluded(&run, &rules).unwrap(), 1);
    store.test_fence(&run, "native-v1:100").unwrap();
    store.complete_initialization(&run).unwrap();
    assert!(store.node("200").is_err());
    assert!(store.node("201").is_err());
}

#[test]
fn prune_with_empty_rules_keeps_every_candidate_node() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    let rules = ExcludeRules::default();
    assert_eq!(store.prune_initialization_excluded(&run, &rules).unwrap(), 0);
    assert!(store.initialization_node(&run, "200").is_ok());
    assert_eq!(
        store
            .initialization_coverage(&run)
            .unwrap()
            .iter()
            .find(|entry| entry.directory == "200")
            .unwrap()
            .state,
        "pending"
    );
}
