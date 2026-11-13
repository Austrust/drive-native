//! Shared task-scoped exclusion rules for issue #50.
//!
//! A profile may carry a list of regular expressions.  They are matched against
//! POSIX paths relative to the task scope root: no leading slash, no `.` or
//! `..` components.  Matching a directory excludes that directory and its
//! whole subtree, so a rule never has to enumerate descendants.

use std::fmt;

use serde::{Deserialize, Serialize};

/// How a matching exclusion pattern is presented to the local mount (#56B).
///
/// `LocalOnly` is the profile default since issue #56: matched paths stay
/// visible and writable locally but never sync. `Hidden` keeps the issue #50
/// behaviour where matched paths answer `ENOENT` and are purged from the local
/// view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExcludeMode {
    /// Visible/usable locally, never uploaded, downloaded or removed remotely.
    #[default]
    LocalOnly,
    /// Issue #50 behaviour: hidden from every local read path and purged.
    Hidden,
}

impl ExcludeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalOnly => "local_only",
            Self::Hidden => "hidden",
        }
    }
}

impl fmt::Display for ExcludeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rejected exclusion pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExcludeError {
    /// The original pattern text, so a settings UI can point at it.
    pub pattern: String,
    /// The regular-expression compiler error.
    pub message: String,
}

impl fmt::Display for ExcludeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid exclude pattern {:?}: {}", self.pattern, self.message)
    }
}

impl std::error::Error for ExcludeError {}

/// Pre-compiled exclusion rules.  Cloning is cheap enough to hand a snapshot to
/// workers; matching never takes a lock.
#[derive(Clone, Debug, Default)]
pub struct ExcludeRules {
    patterns: Vec<regex::Regex>,
}

impl ExcludeRules {
    /// Compile a profile's `exclude_patterns`.  Empty input stays allocation
    /// free and every match is an immediate `false`.
    pub fn compile<S: AsRef<str>>(patterns: &[S]) -> Result<Self, ExcludeError> {
        let mut compiled = Vec::with_capacity(patterns.len());
        for pattern in patterns {
            let source = pattern.as_ref();
            if source.is_empty() {
                return Err(ExcludeError {
                    pattern: source.to_owned(),
                    message: "empty patterns are not allowed".into(),
                });
            }
            match regex::Regex::new(source) {
                Ok(regex) => compiled.push(regex),
                Err(error) => {
                    return Err(ExcludeError {
                        pattern: source.to_owned(),
                        message: error.to_string(),
                    });
                }
            }
        }
        Ok(Self { patterns: compiled })
    }

    /// Stable digest of the compiled pattern sources, used to decide whether a
    /// durable candidate was built under the same rules. Empty rules digest to
    /// the empty string so candidates written before Issue #50 keep resuming.
    pub fn fingerprint(&self) -> String {
        if self.patterns.is_empty() {
            return String::new();
        }
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        for pattern in &self.patterns {
            let source = pattern.as_str();
            hash.update((source.len() as u64).to_le_bytes());
            hash.update(source.as_bytes());
        }
        format!("{:x}", hash.finalize())
    }

    /// True when no rule can ever match.
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// True when `relative_path` (or one of its ancestor directories) matches.
    ///
    /// Directories are also tested with a trailing slash so rules written as
    /// `(^|/)node_modules/` match the directory itself, not only its children.
    pub fn matches(&self, relative_path: &str, is_dir: bool) -> bool {
        if self.patterns.is_empty() {
            return false;
        }
        let path = relative_path.trim_matches('/');
        if path.is_empty() {
            return false;
        }
        if self.matches_node(path, is_dir) {
            return true;
        }
        // A rule matching an ancestor directory excludes every descendant even
        // when the user did not write a subtree expression.
        let mut current = path;
        while let Some((parent, _)) = current.rsplit_once('/') {
            if parent.is_empty() {
                break;
            }
            if self.matches_node(parent, true) {
                return true;
            }
            current = parent;
        }
        false
    }

    fn matches_node(&self, path: &str, is_dir: bool) -> bool {
        self.patterns
            .iter()
            .any(|pattern| pattern.is_match(path) || (is_dir && pattern.is_match(&format!("{path}/"))))
    }
}

/// Convert an absolute path inside the scope to a scope-relative POSIX path.
///
/// The scope root itself maps to the empty string.  Paths outside the scope or
/// containing empty/`.`/`..` components return `None`.
pub fn scope_relative(scope_path: &str, absolute_path: &str) -> Option<String> {
    let scope = scope_path.trim_end_matches('/');
    let rest = if scope.is_empty() {
        absolute_path.trim_start_matches('/')
    } else if absolute_path == scope {
        ""
    } else {
        absolute_path.strip_prefix(scope)?.strip_prefix('/')?
    };
    let rest = rest.trim_end_matches('/');
    if rest.is_empty() {
        return Some(String::new());
    }
    if rest.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        return None;
    }
    Some(rest.to_owned())
}

/// Join a parent relative path with a child name.
pub fn join_relative(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_owned()
    } else {
        format!("{parent}/{name}")
    }
}

/// Opt-in tracing for exclusion decisions. The mount is silent by default so a
/// task without rules pays no logging cost.
fn trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("DRIVE_NATIVE_EXCLUDE_TRACE").is_some_and(|value| value == "1")
    })
}

/// Record one excluded read-path decision when tracing is enabled.
pub fn trace_hit(operation: &str) {
    if trace_enabled() {
        eprintln!("drive-native exclude: {operation} hidden by exclude_patterns");
    }
}

/// Excluded names and subtree inodes for one committed namespace snapshot.
///
/// `entries` holds `(parent_inode, name)` for every directory entry the rules
/// exclude, so readdir/lookup can answer with one hash probe. `inodes` holds
/// every inode in the excluded subtrees, so getattr/open/opendir need no path
/// walk. Both are rebuilt from the store; nothing is persisted.
#[derive(Clone, Debug, Default)]
pub struct ExclusionIndex {
    entries: std::collections::HashSet<(u64, String)>,
    inodes: std::collections::HashSet<u64>,
    /// Node ids of the excluded subtrees. Used by the scheduler defense
    /// (upload/download/cache passes) and by the restart cleanup, which must
    /// cancel local work by node id rather than by inode.
    nodes: std::collections::HashSet<String>,
    /// Node ids of visible directories that directly contain at least one
    /// excluded entry. Renames/moves of such a directory change the scope
    /// relative paths used to match its excluded children, so the pull path
    /// must rebuild the index when one of them is touched.
    parents: std::collections::HashSet<String>,
    /// Scope-relative path of every visible directory inode. New entries below
    /// a directory (create/mkdir/rename) are matched against the full relative
    /// path even though the name does not exist yet; this map makes that lookup
    /// a hash probe during FUSE dispatch.
    directories: std::collections::HashMap<u64, String>,
}

impl ExclusionIndex {
    /// True when no entry and no inode is excluded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.inodes.is_empty()
    }

    /// True when `parent` lists `name` and the rule excludes that entry.
    pub fn entry_excluded(&self, parent_ino: u64, name: &str) -> bool {
        self.entries.contains(&(parent_ino, name.to_owned()))
    }

    /// True when `ino` belongs to an excluded subtree.
    pub fn inode_excluded(&self, ino: u64) -> bool {
        self.inodes.contains(&ino)
    }

    /// Scope-relative path of a visible directory inode.
    pub fn directory_path(&self, ino: u64) -> Option<&str> {
        self.directories.get(&ino).map(String::as_str)
    }

    /// Number of excluded direct entries (diagnostics/tests).
    pub fn entries_len(&self) -> usize {
        self.entries.len()
    }

    /// Number of excluded inodes, including descendants (diagnostics/tests).
    pub fn inodes_len(&self) -> usize {
        self.inodes.len()
    }

    /// True when `node_id` belongs to an excluded subtree.
    pub fn node_excluded(&self, node_id: &str) -> bool {
        self.nodes.contains(node_id)
    }

    /// True when `node_id` is a visible directory with excluded direct
    /// children. Moving such a directory changes those children's paths.
    pub fn parent_has_excluded_children(&self, node_id: &str) -> bool {
        self.parents.contains(node_id)
    }

    /// Excluded node ids (the whole subtree, including the matched entries).
    /// The scheduler defense and the restart cleanup consume this list.
    pub fn excluded_node_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.nodes.iter().cloned().collect();
        ids.sort();
        ids
    }

    /// Number of excluded nodes (diagnostics/tests).
    pub fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    /// Build the index in one pass over the committed namespace.
    ///
    /// The store projection is read while the caller still holds its store
    /// lock; the regular expressions then run on the owned rows without taking
    /// any additional lock.
    pub fn build(store: &crate::Store, root: &str, rules: &ExcludeRules) -> crate::Result<Self> {
        if rules.is_empty() {
            return Ok(Self::default());
        }
        // One projection of the committed namespace. Matching runs on the owned
        // rows: the SQL statement does not take a second lock and the caller's
        // lock is not held by this module beyond the row collection.
        struct Row {
            id: String,
            parent: Option<String>,
            name: String,
            directory: bool,
            ino: u64,
        }
        let mut statement = store.db.prepare_cached(
            "SELECT n.id, n.parent, n.name, n.directory, i.ino \
             FROM nodes n JOIN fs_inodes i ON i.node = n.id \
             WHERE n.deleted = 0 ORDER BY n.id",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok(Row {
                    id: row.get(0)?,
                    parent: row.get(1)?,
                    name: row.get(2)?,
                    directory: row.get(3)?,
                    ino: {
                        let ino: i64 = row.get(4)?;
                        u64::try_from(ino)
                            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(4, ino))?
                    },

                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let root_row = rows
            .iter()
            .find(|row| row.id == root)
            .ok_or_else(|| crate::Error::State("mount root is missing from the namespace".into()))?;
        if !root_row.directory {
            return Err(crate::Error::State("mount root is not a directory".into()));
        }
        let root_ino = root_row.ino;
        let mut children: std::collections::HashMap<&str, Vec<usize>> =
            std::collections::HashMap::new();
        for (index, row) in rows.iter().enumerate() {
            if let Some(parent) = row.parent.as_deref() {
                children.entry(parent).or_default().push(index);
            }
        }
        let mut index = Self::default();
        index.directories.insert(root_ino, String::new());
        let mut queue: Vec<(&str, String, u64)> = vec![(root, String::new(), root_ino)];
        while let Some((directory, relative, ino)) = queue.pop() {
            let Some(kids) = children.get(directory) else {
                continue;
            };
            for &kid in kids {
                let row = &rows[kid];
                let child_relative = join_relative(&relative, &row.name);
                if rules.matches(&child_relative, row.directory) {
                    index.entries.insert((ino, row.name.clone()));
                    index.parents.insert(directory.to_owned());
                    // Exclude the whole subtree: every descendant inode must
                    // answer ENOENT even without a path walk.
                    let mut pending = vec![kid];
                    while let Some(current) = pending.pop() {
                        let row = &rows[current];
                        index.inodes.insert(row.ino);
                        index.nodes.insert(row.id.clone());
                        if let Some(grandchildren) = children.get(row.id.as_str()) {
                            pending.extend(grandchildren.iter().copied());
                        }
                    }
                } else if row.directory {
                    index
                        .directories
                        .insert(row.ino, child_relative.clone());
                    queue.push((row.id.as_str(), child_relative, row.ino));
                }
            }
        }
        if !index.is_empty() && trace_enabled() {
            eprintln!(
                "drive-native exclude: index built with {} entries and {} inodes",
                index.entries.len(),
                index.inodes.len()
            );
        }
        Ok(index)
    }
}

/// Rules plus the current exclusion index, shared between the FUSE mount and
/// the sync bridge. Reads only take a short `RwLock` and clone the `Arc`.
pub struct ExclusionState {
    rules: ExcludeRules,
    mode: ExcludeMode,
    index: std::sync::RwLock<std::sync::Arc<ExclusionIndex>>,
    /// Node ids that were created or written under a `local_only` rule in this
    /// session. The committed-namespace index covers them after a refresh; this
    /// set closes the window between a FUSE write and the next index rebuild so
    /// the scheduler never starts an upload for them.
    local_only_nodes: std::sync::RwLock<std::collections::HashSet<String>>,
}

impl ExclusionState {
    /// Create state for a task with the legacy issue #50 (`hidden`) behaviour.
    ///
    /// Production profiles go through [`Self::with_mode`]; this constructor is
    /// kept for callers and tests that predate #56B. An empty rule list leaves
    /// the index empty and every fast-path check collapsed to one boolean read.
    pub fn new(rules: ExcludeRules) -> Self {
        Self::with_mode(rules, ExcludeMode::Hidden)
    }

    /// Create state for a task with an explicit profile mode.
    pub fn with_mode(rules: ExcludeRules, mode: ExcludeMode) -> Self {
        Self {
            rules,
            mode,
            index: std::sync::RwLock::new(std::sync::Arc::new(ExclusionIndex::default())),
            local_only_nodes: std::sync::RwLock::new(std::collections::HashSet::new()),
        }
    }

    /// The profile-selected mode; `hidden` is an explicit opt-in.
    pub fn mode(&self) -> ExcludeMode {
        self.mode
    }

    /// True when the mode hides matched paths from the local view (#50).
    pub fn hides(&self) -> bool {
        self.mode == ExcludeMode::Hidden
    }

    /// Record a node created or written under a `local_only` rule so the
    /// scheduler refuses it before the next full index rebuild.
    pub fn mark_local_only_node(&self, node_id: &str) {
        self.local_only_nodes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(node_id.to_owned());
    }

    /// True when `node_id` was marked local-only in this session.
    pub fn is_local_only_node(&self, node_id: &str) -> bool {
        self.local_only_nodes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(node_id)
    }

    /// The compiled rules (used by the initialization owner and the bridge).
    pub fn rules(&self) -> &ExcludeRules {
        &self.rules
    }

    /// True when at least one rule is configured.
    pub fn is_active(&self) -> bool {
        !self.rules.is_empty()
    }

    /// Current index snapshot. Cheap: one `RwLock` read plus an `Arc` clone.
    pub fn current(&self) -> std::sync::Arc<ExclusionIndex> {
        self.index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// True when a not-yet-created entry below `parent_ino` matches the rules.
    ///
    /// Write paths call this before recording anything locally. The parent's
    /// scope-relative path comes from the index, so the full path is matched
    /// without taking the store lock during FUSE dispatch.
    pub fn child_matches(
        &self,
        index: &ExclusionIndex,
        parent_ino: u64,
        name: &str,
        is_dir: bool,
    ) -> bool {
        if self.rules.is_empty() {
            return false;
        }
        let Some(parent) = index.directory_path(parent_ino) else {
            return false;
        };
        self.rules.matches(&join_relative(parent, name), is_dir)
    }

    /// Rebuild the index from the committed namespace. Called at mount time and
    /// after every namespace swap (initialization install, recovery snapshot,
    /// applied pull) so the index never describes a superseded snapshot.
    pub fn refresh(&self, store: &crate::Store, root: &str) -> crate::Result<()> {
        if !self.is_active() {
            *self
                .index
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                std::sync::Arc::new(ExclusionIndex::default());
            return Ok(());
        }
        let index = ExclusionIndex::build(store, root, &self.rules)?;
        *self
            .index
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = std::sync::Arc::new(index);
        Ok(())
    }

    /// `refresh` for a shared store boundary; takes and releases the store lock.
    pub fn refresh_shared(
        &self,
        store: &crate::SharedStore,
        root: &str,
    ) -> crate::Result<()> {
        if !self.is_active() {
            return Ok(());
        }
        let guard = store
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.refresh(&guard, root)
    }
}

/// Resolve the scope-relative path of `id` by walking its parent chain.
///
/// Returns `(relative_path, is_directory)`; the scope root resolves to `("",
/// true)`. `None` means the node or one of its ancestors is unknown, which the
/// event filter treats as excluded (an unresolvable event cannot be classified
/// as visible).
fn relative_path_of(
    root: &str,
    id: &str,
    mut lookup: impl FnMut(&str) -> Option<crate::Node>,
) -> Option<(String, bool)> {
    let mut names: Vec<String> = Vec::new();
    let mut current = id.to_owned();
    let mut directory = true;
    for _ in 0..256 {
        if current == root {
            let relative = names.iter().rev().cloned().collect::<Vec<_>>().join("/");
            return Some((relative, directory));
        }
        let node = lookup(&current)?;
        directory = node.directory;
        names.push(node.name);
        current = node.parent?;
    }
    None
}

/// Drop snapshot nodes whose scope-relative path is excluded. Returns the
/// number of dropped nodes. The root itself is never excluded.
pub fn retain_unexcluded_snapshot_nodes(
    scope_file_id: &str,
    rules: &ExcludeRules,
    nodes: &mut Vec<crate::Node>,
) -> usize {
    if rules.is_empty() {
        return 0;
    }
    let by_id: std::collections::HashMap<String, crate::Node> = nodes
        .iter()
        .cloned()
        .map(|node| (node.id.clone(), node))
        .collect();
    let mut kept = Vec::with_capacity(nodes.len());
    let mut dropped = 0usize;
    for node in nodes.iter() {
        if node.id == scope_file_id {
            kept.push(node.clone());
            continue;
        }
        let resolved = relative_path_of(scope_file_id, &node.id, |id| by_id.get(id).cloned());
        match resolved {
            Some((relative, is_dir)) if rules.matches(&relative, is_dir) => {
                dropped += 1;
                trace_hit("snapshot node");
            }
            _ => kept.push(node.clone()),
        }
    }
    *nodes = kept;
    dropped
}

/// Drop pull events whose node path is excluded, keeping parent/child
/// consistency: a child of a dropped parent is dropped as well even when its
/// own name does not match. Uses the same-batch upsert map before touching the
/// store, then the committed nodes for events not in the batch.
pub fn retain_unexcluded_events(
    store: &crate::Store,
    root: &str,
    rules: &ExcludeRules,
    events: &mut Vec<crate::Event>,
) -> usize {
    if rules.is_empty() {
        return 0;
    }
    // New directories and files can arrive in one batch before any of them is
    // committed; resolve those parent links from the batch itself first.
    let batch: std::collections::HashMap<String, crate::Node> = events
        .iter()
        .filter_map(|event| match &event.change {
            crate::Change::Upsert(node) => Some((node.id.clone(), node.clone())),
            crate::Change::Delete { .. } => None,
        })
        .collect();
    let mut kept = Vec::with_capacity(events.len());
    let mut dropped = 0usize;
    for event in events.drain(..) {
        let id = match &event.change {
            crate::Change::Upsert(node) => node.id.as_str(),
            crate::Change::Delete { id } => id.as_str(),
        };
        let resolved = relative_path_of(root, id, |id| {
            batch
                .get(id)
                .cloned()
                .or_else(|| store.node(id).ok())
        });
        let excluded = match resolved {
            Some((relative, is_dir)) => rules.matches(&relative, is_dir),
            None => true,
        };
        if excluded {
            dropped += 1;
            trace_hit("pull event");
        } else {
            kept.push(event);
        }
    }
    *events = kept;
    dropped
}

/// True when applying `events` could change the exclusion index.
///
/// The pull path rebuilds the index only when this returns true. Excluded
/// events never reach the committed namespace (`retain_unexcluded_events` drops
/// them first), so a normal batch cannot change exclusion visibility; the walk
/// below detects the remaining cases that can: a touched node is itself
/// excluded or indexed, a touched visible directory directly contains excluded
/// entries (a rename changes their relative paths), or the node's path matches
/// the rules. Unresolvable nodes are treated conservatively as affecting the
/// index.
pub fn events_may_affect_index(
    store: &crate::Store,
    root: &str,
    rules: &ExcludeRules,
    index: &ExclusionIndex,
    events: &[crate::Event],
) -> bool {
    if rules.is_empty() || events.is_empty() {
        return false;
    }
    let batch: std::collections::HashMap<String, crate::Node> = events
        .iter()
        .filter_map(|event| match &event.change {
            crate::Change::Upsert(node) => Some((node.id.clone(), node.clone())),
            crate::Change::Delete { .. } => None,
        })
        .collect();
    for event in events {
        let id = match &event.change {
            crate::Change::Upsert(node) => node.id.as_str(),
            crate::Change::Delete { id } => id.as_str(),
        };
        if event_touches_exclusion(store, &batch, root, rules, index, id) {
            return true;
        }
    }
    false
}

fn event_touches_exclusion(
    store: &crate::Store,
    batch: &std::collections::HashMap<String, crate::Node>,
    root: &str,
    rules: &ExcludeRules,
    index: &ExclusionIndex,
    id: &str,
) -> bool {
    let mut names = Vec::new();
    let mut directory = true;
    let mut current = id.to_owned();
    for _ in 0..256 {
        if index.node_excluded(&current) || index.parent_has_excluded_children(&current) {
            return true;
        }
        if current == root {
            let relative = names.iter().rev().cloned().collect::<Vec<_>>().join("/");
            return rules.matches(&relative, directory);
        }
        let Some(node) = batch
            .get(&current)
            .cloned()
            .or_else(|| store.node(&current).ok())
        else {
            return true;
        };
        if names.is_empty() {
            directory = node.directory;
        }
        names.push(node.name);
        let Some(parent) = node.parent else {
            return true;
        };
        current = parent;
    }
    true
}

#[cfg(test)]
#[path = "exclude_index_tests.rs"]
mod index_tests;

#[cfg(test)]
#[path = "exclude_local_only_tests.rs"]
mod local_only_tests;

#[cfg(test)]
#[path = "exclude_purge_tests.rs"]
mod purge_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(patterns: &[&str]) -> ExcludeRules {
        ExcludeRules::compile(patterns).unwrap()
    }

    #[test]
    fn empty_rules_never_match_and_report_empty() {
        let empty = ExcludeRules::default();
        assert!(empty.is_empty());
        assert!(!empty.matches("node_modules", true));
        assert!(!empty.matches("a.iso", false));
        assert!(!empty.matches("", false));
        let compiled = rules(&[]);
        assert!(compiled.is_empty());
        assert!(!compiled.matches("anything", true));
    }

    #[test]
    fn file_rules_match_files_and_directories_with_the_same_name() {
        let rules = rules(&[r"\.iso$"]);
        assert!(rules.matches("install.iso", false));
        assert!(rules.matches("dir/install.iso", false));
        assert!(!rules.matches("install.iso.txt", false));
        // A directory entry named like a file still matches: the whole subtree
        // disappears.
        assert!(rules.matches("install.iso", true));
    }

    #[test]
    fn directory_rule_with_trailing_slash_matches_the_directory_itself() {
        let rules = rules(&[r"(^|/)node_modules/"]);
        assert!(rules.matches("node_modules", true));
        assert!(rules.matches("node_modules/inner.js", false));
        assert!(rules.matches("x/node_modules", true));
        assert!(!rules.matches("node_modules2", true));
        assert!(!rules.matches("node_modules2/inner.js", false));
        // A *file* named node_modules is not a directory, so the slash rule
        // cannot match it.
        assert!(!rules.matches("node_modules", false));
    }

    #[test]
    fn ancestor_match_excludes_every_descendant() {
        let rules = rules(&[r"^Pinned$"]);
        assert!(rules.matches("Pinned", true));
        assert!(rules.matches("Pinned/deep/file.txt", false));
        assert!(rules.matches("Pinned/deep/dir", true));
        assert!(!rules.matches("NotPinned", true));
        assert!(!rules.matches("PinnedFile", true));
    }

    #[test]
    fn nested_anchor_only_matches_exact_ancestor_component() {
        let rules = rules(&[r"^a/b$"]);
        assert!(rules.matches("a/b", true));
        assert!(rules.matches("a/b/c/d", false));
        assert!(!rules.matches("a/bc/d", false));
        assert!(!rules.matches("a", true));
    }

    #[test]
    fn matching_is_case_sensitive_unless_the_user_writes_an_inline_flag() {
        let bare = rules(&[r"^Docs$"]);
        assert!(!bare.matches("docs", true));
        assert!(bare.matches("Docs", true));
        let insensitive = rules(&[r"(?i)^Docs$"]);
        assert!(insensitive.matches("docs", true));
        assert!(insensitive.matches("DOcs/deep", false));
    }

    #[test]
    fn empty_dot_and_root_paths_are_never_excluded() {
        let rules = rules(&[r"^$", r".*"]);
        assert!(!rules.matches("", true));
        assert!(!rules.matches("", false));
        assert!(!rules.matches("/", true));
        // A `.*` rule still matches real relative paths.
        assert!(rules.matches("file.txt", false));
    }

    #[test]
    fn surrounding_slashes_are_normalized() {
        let rules = rules(&[r"^dir/file\.txt$"]);
        assert!(rules.matches("dir/file.txt", false));
        assert!(rules.matches("/dir/file.txt", false));
        assert!(rules.matches("dir/file.txt/", false));
    }

    #[test]
    fn invalid_pattern_reports_the_pattern_and_the_compiler_error() {
        let error = ExcludeRules::compile(&["ok", "[unclosed"]).unwrap_err();
        assert_eq!(error.pattern, "[unclosed");
        assert!(!error.message.is_empty());
        let text = error.to_string();
        assert!(text.contains("[unclosed"), "{text}");
        assert!(text.contains("invalid exclude pattern"), "{text}");
    }

    #[test]
    fn empty_pattern_is_rejected_instead_of_matching_everything() {
        let error = ExcludeRules::compile(&[""]).unwrap_err();
        assert_eq!(error.pattern, "");
        assert!(!error.message.is_empty());
    }

    #[test]
    fn cloned_rules_match_like_the_original() {
        let original = rules(&[r"(^|/)\.tmp-", r"\.iso$"]);
        let clone = original.clone();
        for (path, is_dir) in [
            ("a.iso", false),
            ("x/a.iso", false),
            (".tmp-build", true),
            ("x/.tmp-build", true),
            ("a.txt", false),
        ] {
            assert_eq!(original.matches(path, is_dir), clone.matches(path, is_dir), "{path}");
        }
        assert!(clone.matches(".tmp-build/cache", false));
    }

    #[test]
    fn scope_relative_strips_only_the_scope_prefix() {
        assert_eq!(scope_relative("/Lab", "/Lab"), Some(String::new()));
        assert_eq!(scope_relative("/Lab", "/Lab/Sub"), Some("Sub".into()));
        assert_eq!(scope_relative("/Lab", "/Lab/Sub/C.txt"), Some("Sub/C.txt".into()));
        assert_eq!(scope_relative("/Lab", "/LabX"), None);
        assert_eq!(scope_relative("/Lab", "/Other"), None);
        assert_eq!(scope_relative("/", "/"), Some(String::new()));
        assert_eq!(scope_relative("/", "/a/b"), Some("a/b".into()));
        assert_eq!(scope_relative("/Lab", "/Lab/a//b"), None);
        assert_eq!(scope_relative("/Lab", "/Lab/../etc"), None);
    }

    #[test]
    fn join_relative_skips_the_separator_at_the_scope_root() {
        assert_eq!(join_relative("", "Sub"), "Sub");
        assert_eq!(join_relative("Sub", "C.txt"), "Sub/C.txt");
    }
}
