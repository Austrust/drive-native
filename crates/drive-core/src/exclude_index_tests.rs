//! Issue #50 phase 3: exclusion index and namespace-filter tests.
use super::*;
use crate::{Change, Event, Node, Store};

struct Fixture {
    _dir: tempfile::TempDir,
    store: Store,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::open(dir.path()).expect("open store");
    Fixture { _dir: dir, store }
}

fn insert_node(store: &Store, id: &str, parent: Option<&str>, name: &str, directory: bool) {
    store
        .db
        .execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES(?1,?2,?3,'v1',?4)",
            rusqlite::params![id, parent, name, directory],
        )
        .expect("insert node");
}

fn allocate_inodes(store: &Store, root: &str) {
    // Tests mutate the namespace directly; clear the one-time completeness
    // marker so `ensure_filesystem_inodes` also covers nodes inserted later.
    store
        .db
        .execute("DELETE FROM metadata WHERE key='fs_inodes_complete'", [])
        .expect("reset inode marker");
    store.ensure_fs_root(root).expect("root inode");
    store
        .ensure_filesystem_inodes(root)
        .expect("namespace inodes");
}

fn ino(store: &Store, id: &str) -> u64 {
    let ino: i64 = store
        .db
        .query_row("SELECT ino FROM fs_inodes WHERE node=?1", [id], |row| {
            row.get(0)
        })
        .expect("inode");
    u64::try_from(ino).expect("positive inode")
}

fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
    Node {
        id: id.to_owned(),
        parent: parent.map(str::to_owned),
        name: name.to_owned(),
        version: "v1".into(),
        directory,
        metadata: None,
    }
}

fn upsert(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Event {
    Event {
        id: id.to_owned(),
        change: Change::Upsert(node(id, parent, name, directory)),
    }
}

fn delete(id: &str) -> Event {
    Event {
        id: id.to_owned(),
        change: Change::Delete { id: id.to_owned() },
    }
}

fn rules(patterns: &[&str]) -> ExcludeRules {
    ExcludeRules::compile(patterns).expect("valid rules")
}

#[test]
fn excluded_entries_and_subtree_inodes_are_indexed() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "101", Some("100"), "A.txt", false);
    insert_node(store, "200", Some("100"), "node_modules", true);
    insert_node(store, "201", Some("200"), "left-pad", true);
    insert_node(store, "202", Some("201"), "index.js", false);
    insert_node(store, "300", Some("100"), "Pinned", true);
    insert_node(store, "301", Some("300"), "keep.txt", false);
    allocate_inodes(store, "100");

    let index = ExclusionIndex::build(
        store,
        "100",
        &rules(&[r"(^|/)node_modules/", r"^Pinned$"]),
    )
    .expect("build index");

    assert_eq!(index.entries_len(), 2, "two direct children are excluded");
    assert!(index.entry_excluded(ino(store, "100"), "node_modules"));
    assert!(index.entry_excluded(ino(store, "100"), "Pinned"));
    assert!(!index.entry_excluded(ino(store, "100"), "A.txt"));
    for id in ["200", "201", "202", "300", "301"] {
        assert!(index.inode_excluded(ino(store, id)), "{id} must be excluded");
    }
    for id in ["100", "101"] {
        assert!(!index.inode_excluded(ino(store, id)), "{id} stays visible");
    }
}

#[test]
fn nested_directory_rules_use_paths_relative_to_the_scope_root() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "110", Some("100"), "src", true);
    insert_node(store, "111", Some("110"), "node_modules", true);
    insert_node(store, "112", Some("111"), "x.js", false);
    allocate_inodes(store, "100");

    let index =
        ExclusionIndex::build(store, "100", &rules(&[r"(^|/)node_modules/"])).expect("index");

    assert_eq!(index.entries_len(), 1);
    assert!(index.entry_excluded(ino(store, "110"), "node_modules"));
    assert!(index.inode_excluded(ino(store, "111")));
    assert!(index.inode_excluded(ino(store, "112")));
    assert!(!index.entry_excluded(ino(store, "100"), "src"));
}

#[test]
fn empty_rules_build_an_empty_index() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "200", Some("100"), "node_modules", true);
    allocate_inodes(store, "100");

    let index =
        ExclusionIndex::build(store, "100", &ExcludeRules::default()).expect("build index");

    assert!(index.is_empty());
    assert_eq!(index.entries_len(), 0);
    assert_eq!(index.inodes_len(), 0);
}

#[test]
fn refresh_rebuilds_the_index_after_namespace_changes() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    allocate_inodes(store, "100");

    let state = ExclusionState::new(rules(&[r"(^|/)node_modules/"]));
    state.refresh(store, "100").expect("initial refresh");
    assert!(state.current().is_empty(), "nothing excluded yet");

    // Simulated namespace swap: a new excluded directory appears.
    insert_node(store, "200", Some("100"), "node_modules", true);
    insert_node(store, "201", Some("200"), "x.js", false);
    allocate_inodes(store, "100");
    state.refresh(store, "100").expect("refresh after swap");
    let index = state.current();
    assert_eq!(index.entries_len(), 1);
    assert!(index.inode_excluded(ino(store, "201")));

    // And it disappears again after the namespace is replaced.
    store
        .db
        .execute("DELETE FROM fs_inodes WHERE node IN ('200','201')", [])
        .expect("drop inodes");
    store
        .db
        .execute("DELETE FROM nodes WHERE id IN ('200','201')", [])
        .expect("drop nodes");
    state.refresh(store, "100").expect("refresh after removal");
    assert!(state.current().is_empty());
}

#[test]
fn refresh_with_empty_rules_never_queries_the_namespace() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "200", Some("100"), "node_modules", true);
    allocate_inodes(store, "100");

    let state = ExclusionState::new(ExcludeRules::default());
    state.refresh(store, "100").expect("no-op refresh");
    assert!(!state.is_active());
    assert!(state.current().is_empty());
}

#[test]
fn snapshot_filter_drops_excluded_subtree_and_keeps_siblings() {
    let mut nodes = vec![
        node("100", None, "Lab", true),
        node("101", Some("100"), "A.txt", false),
        node("200", Some("100"), "node_modules", true),
        node("201", Some("200"), "index.js", false),
        node("300", Some("100"), "Pinned", true),
        node("301", Some("300"), "keep.txt", false),
    ];

    let dropped = retain_unexcluded_snapshot_nodes(
        "100",
        &rules(&[r"(^|/)node_modules/", r"^Pinned$"]),
        &mut nodes,
    );

    assert_eq!(dropped, 4, "the two directories and their files disappear");
    let ids: Vec<&str> = nodes.iter().map(|node| node.id.as_str()).collect();
    assert_eq!(ids, ["100", "101"]);
}

#[test]
fn snapshot_filter_never_drops_the_scope_root() {
    let mut nodes = vec![node("100", None, "Lab", true)];
    let dropped = retain_unexcluded_snapshot_nodes("100", &rules(&[r".*"]), &mut nodes);
    assert_eq!(dropped, 0);
    assert_eq!(nodes.len(), 1);
}

#[test]
fn event_filter_drops_children_of_excluded_parents_inside_one_batch() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "200", Some("100"), "Pinned", true);
    insert_node(store, "201", Some("100"), "Other", true);
    allocate_inodes(store, "100");
    let mut events = vec![
        upsert("200", Some("100"), "Pinned", true),
        upsert("300", Some("200"), "new.txt", false),
        upsert("201", Some("100"), "Other", true),
    ];

    let dropped = retain_unexcluded_events(store, "100", &rules(&[r"^Pinned$"]), &mut events);

    assert_eq!(dropped, 2);
    let ids: Vec<&str> = events.iter().map(|event| event.id.as_str()).collect();
    assert_eq!(ids, ["201"]);
}

#[test]
fn event_filter_drops_a_nonmatching_child_of_an_excluded_batch_parent() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    allocate_inodes(store, "100");
    let mut events = vec![
        upsert("200", Some("100"), "Pinned", true),
        upsert("300", Some("200"), "keep.txt", false),
    ];

    // The child's own name matches nothing, but its new parent is excluded.
    let dropped = retain_unexcluded_events(store, "100", &rules(&[r"^Pinned$"]), &mut events);

    assert_eq!(dropped, 2);
    assert!(events.is_empty());
}

#[test]
fn event_filter_drops_an_excluded_child_and_keeps_its_parent() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "201", Some("100"), "Other", true);
    allocate_inodes(store, "100");
    let mut events = vec![
        upsert("201", Some("100"), "Other", true),
        upsert("400", Some("201"), "install.iso", false),
    ];

    let dropped = retain_unexcluded_events(store, "100", &rules(&[r"\.iso$"]), &mut events);

    assert_eq!(dropped, 1);
    let ids: Vec<&str> = events.iter().map(|event| event.id.as_str()).collect();
    assert_eq!(ids, ["201"]);
}

#[test]
fn event_filter_resolves_nested_paths_for_batched_new_directories() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    allocate_inodes(store, "100");
    let mut events = vec![
        upsert("110", Some("100"), "src", true),
        upsert("111", Some("110"), "node_modules", true),
        upsert("112", Some("111"), "x.js", false),
    ];

    let dropped =
        retain_unexcluded_events(store, "100", &rules(&[r"(^|/)node_modules/"]), &mut events);

    assert_eq!(dropped, 2);
    let ids: Vec<&str> = events.iter().map(|event| event.id.as_str()).collect();
    assert_eq!(ids, ["110"]);
}

#[test]
fn event_filter_drops_deletes_belonging_to_excluded_subtrees() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "200", Some("100"), "Pinned", true);
    insert_node(store, "300", Some("200"), "file.txt", false);
    allocate_inodes(store, "100");
    let mut events = vec![delete("200"), delete("300"), delete("missing")];

    let dropped = retain_unexcluded_events(store, "100", &rules(&[r"^Pinned$"]), &mut events);

    assert_eq!(dropped, 3);
    assert!(events.is_empty());
}

#[test]
fn event_filter_keeps_unrelated_events() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "101", Some("100"), "A.txt", false);
    allocate_inodes(store, "100");
    let mut events = vec![
        upsert("101", Some("100"), "A.txt", false),
        upsert("102", Some("100"), "B.txt", false),
    ];

    let dropped = retain_unexcluded_events(store, "100", &rules(&[r"\.iso$"]), &mut events);

    assert_eq!(dropped, 0);
    assert_eq!(events.len(), 2);
}

#[test]
fn empty_rules_never_drop_events_or_snapshot_nodes() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "200", Some("100"), "node_modules", true);
    allocate_inodes(store, "100");
    let mut events = vec![
        upsert("200", Some("100"), "node_modules", true),
        upsert("201", Some("200"), "x.js", false),
    ];
    let mut nodes = vec![
        node("100", None, "Lab", true),
        node("200", Some("100"), "node_modules", true),
    ];
    let empty = ExcludeRules::default();

    assert_eq!(retain_unexcluded_events(store, "100", &empty, &mut events), 0);
    assert_eq!(events.len(), 2);
    assert_eq!(
        retain_unexcluded_snapshot_nodes("100", &empty, &mut nodes),
        0
    );
    assert_eq!(nodes.len(), 2);
}

#[test]
fn event_gate_skips_batches_that_cannot_change_exclusions() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "101", Some("100"), "A.txt", false);
    insert_node(store, "110", Some("100"), "src", true);
    allocate_inodes(store, "100");
    let rules = rules(&[r"(^|/)node_modules/"]);
    let index = ExclusionIndex::build(store, "100", &rules).expect("index");
    let events = vec![
        upsert("101", Some("100"), "A.txt", false),
        upsert("110", Some("100"), "src", true),
    ];

    assert!(!events_may_affect_index(
        store, "100", &rules, &index, &events
    ));
}

#[test]
fn event_gate_detects_excluded_paths_and_indexed_nodes() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "200", Some("100"), "Pinned", true);
    insert_node(store, "201", Some("200"), "keep.txt", false);
    allocate_inodes(store, "100");
    let rules = rules(&[r"^Pinned$"]);
    let index = ExclusionIndex::build(store, "100", &rules).expect("index");
    assert!(index.parent_has_excluded_children("100"));

    // A delete of a node that is currently indexed changes the index.
    assert!(events_may_affect_index(
        store,
        "100",
        &rules,
        &index,
        &[delete("200")],
    ));
    // An event whose resolved path matches the rules changes the index too.
    assert!(events_may_affect_index(
        store,
        "100",
        &rules,
        &index,
        &[upsert("201", Some("200"), "keep.txt", false)],
    ));
}

#[test]
fn event_gate_detects_renames_of_directories_with_excluded_children() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    insert_node(store, "110", Some("100"), "src", true);
    insert_node(store, "111", Some("110"), "node_modules", true);
    insert_node(store, "112", Some("111"), "x.js", false);
    allocate_inodes(store, "100");
    let rules = rules(&[r"(^|/)node_modules/"]);
    let index = ExclusionIndex::build(store, "100", &rules).expect("index");
    assert!(index.parent_has_excluded_children("110"));

    // Renaming "src" changes the relative paths of its excluded children even
    // though "src" itself does not match the rules.
    assert!(events_may_affect_index(
        store,
        "100",
        &rules,
        &index,
        &[upsert("110", Some("100"), "renamed", true)],
    ));
}

#[test]
fn event_gate_is_conservative_for_unresolvable_events() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    allocate_inodes(store, "100");
    let rules = rules(&[r"\.iso$"]);
    let index = ExclusionIndex::build(store, "100", &rules).expect("index");

    assert!(events_may_affect_index(
        store,
        "100",
        &rules,
        &index,
        &[upsert("999", Some("missing-parent"), "x.iso", false)],
    ));
}

#[test]
fn empty_rules_or_events_never_affect_the_index() {
    let fixture = fixture();
    let store = &fixture.store;
    insert_node(store, "100", None, "Lab", true);
    allocate_inodes(store, "100");
    let index = ExclusionIndex::default();
    let events = vec![upsert("101", Some("100"), "x.iso", false)];
    let empty = ExcludeRules::default();

    assert!(!events_may_affect_index(
        store, "100", &empty, &index, &events
    ));
    assert!(!events_may_affect_index(
        store,
        "100",
        &rules(&[r"\.iso$"]),
        &index,
        &[],
    ));
}
