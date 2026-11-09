//! Shared task-scoped exclusion rules for issue #50.
//!
//! A profile may carry a list of regular expressions.  They are matched against
//! POSIX paths relative to the task scope root: no leading slash, no `.` or
//! `..` components.  Matching a directory excludes that directory and its
//! whole subtree, so a rule never has to enumerate descendants.

use std::fmt;

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
