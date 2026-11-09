#!/usr/bin/env bash
# check-core-posix.sh — guard crates/drive-core against new POSIX-only code.
#
# Why: issue #11 requires drive-core to stay buildable without POSIX-only
# dependencies so a Windows adapter can be added without moving the core's
# business rules. This script is the cheap regression gate for that rule.
#
# Scope:
#   crates/drive-core/src/**/*.rs   production sources
#   crates/drive-core/Cargo.toml    platform-only dependencies (libc/nix)
#   crates/drive-core/build.rs      scanned with the Rust rules when present
#
# Test modules (files ending in `_tests.rs`) are out of scope: they run on
# Linux development hosts and are not part of the portable core surface.
# Inline `#[cfg(test)]` blocks inside production files still show up as hits
# and must be allowlisted explicitly, so the allowlist documents them too.
#
# Usage:
#   bash scripts/check-core-posix.sh            # report violations; exit 1 if any
#   bash scripts/check-core-posix.sh --list     # list every hit and its status
#   bash scripts/check-core-posix.sh --strict   # ignore the allowlist; exit 1 if any hit
#   bash scripts/check-core-posix.sh --help
#
# Allowlist: scripts/core-posix-allowlist.txt
#   One rule per line: <repo-relative path>:<extended-regexp>
#   The regex is matched against the hitting line. `#` starts a comment line;
#   every allowlisted construct carries a "why" and, where applicable, the
#   migration step that will delete the entry.
#
# Exit codes: 0 no violations, 1 violations, 2 usage/environment error.
#
# Rules (deliberately narrow to avoid cross-platform false positives):
#   unix_api   std::os::unix imports/APIs (including multi-line `os::unix::...`)
#   libc       libc:: calls (errno construction, POSIX-only flags, etc.)
#   nix        nix crate calls (word-boundary: `unix::` must not match)
#   cfg_unix   #[cfg(unix)] gates, so platform-only blocks stay documented
#   dir_fsync  File::open(<dir>)?.sync_all() — opening a directory is POSIX-only
#   posix_dep  libc/nix dependencies in drive-core/Cargo.toml
#
# Not flagged on purpose: std::fs::symlink_metadata / FileType::Symlink (cross
# platform), comments that merely contain the word "chmod", and file fsync on
# regular files (works on Windows).

set -uo pipefail

usage() {
    sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'
}

mode="default"
case "${1:-}" in
    "") ;;
    --list) mode="list" ;;
    --strict) mode="strict" ;;
    -h|--help) usage; exit 0 ;;
    *)
        echo "check-core-posix: unknown option: $1" >&2
        usage >&2
        exit 2
        ;;
esac
if [[ $# -gt 1 ]]; then
    echo "check-core-posix: expected at most one argument, got: $*" >&2
    exit 2
fi

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/.." && pwd)
core_dir="$repo_root/crates/drive-core"
src_dir="$core_dir/src"
cargo_toml="$core_dir/Cargo.toml"
build_rs="$core_dir/build.rs"
allowlist="$script_dir/core-posix-allowlist.txt"

if [[ ! -d "$src_dir" ]]; then
    echo "check-core-posix: missing directory: $src_dir" >&2
    exit 2
fi

RULE_NAMES=(unix_api libc nix cfg_unix dir_fsync)
declare -A RULE_REGEX=(
    [unix_api]='(^|[^A-Za-z0-9_])os::unix(::|_)'
    [libc]='(^|[^A-Za-z0-9_])libc::'
    [nix]='(^|[^A-Za-z0-9_])nix::'
    [cfg_unix]='#\[cfg\(unix\)\]'
    [dir_fsync]='File::open\(.*\)\?[[:space:]]*\.sync_all\(\)'
)
CARGO_RULE_NAME=posix_dep
CARGO_RULE_REGEX='^[[:space:]]*(libc|nix)[[:space:]]*='

declare -A HIT_RULE=()
declare -A HIT_TEXT=()

record_hits() {
    # record_hits <relpath> <file> <rule> <ere>
    local rel="$1" file="$2" rule="$3" regex="$4" out line_no content key
    while IFS= read -r out; do
        [[ -n "$out" ]] || continue
        line_no="${out%%:*}"
        content="${out#*:}"
        key="$rel:$line_no"
        if [[ -z "${HIT_RULE[$key]+x}" ]]; then
            HIT_RULE[$key]="$rule"
            HIT_TEXT[$key]="$content"
        fi
    done < <(grep -nE -- "$regex" "$file" 2>/dev/null || true)
}

scanned=0
shopt -s globstar nullglob
for file in "$src_dir"/**/*.rs; do
    [[ -f "$file" ]] || continue
    base="${file##*/}"
    [[ "$base" == *_tests.rs ]] && continue
    rel="crates/drive-core/src/${file#"$src_dir"/}"
    scanned=$((scanned + 1))
    for rule in "${RULE_NAMES[@]}"; do
        record_hits "$rel" "$file" "$rule" "${RULE_REGEX[$rule]}"
    done
done

if [[ -f "$cargo_toml" ]]; then
    scanned=$((scanned + 1))
    record_hits "crates/drive-core/Cargo.toml" "$cargo_toml" "$CARGO_RULE_NAME" "$CARGO_RULE_REGEX"
fi
if [[ -f "$build_rs" ]]; then
    scanned=$((scanned + 1))
    for rule in "${RULE_NAMES[@]}"; do
        record_hits "crates/drive-core/build.rs" "$build_rs" "$rule" "${RULE_REGEX[$rule]}"
    done
fi

AL_PATH=()
AL_REGEX=()
allowlist_entries=0
if [[ "$mode" != "strict" && ! -f "$allowlist" ]]; then
    echo "check-core-posix: warning: allowlist not found: $allowlist" >&2
fi
if [[ "$mode" != "strict" && -f "$allowlist" ]]; then
    while IFS= read -r raw || [[ -n "$raw" ]]; do
        line="${raw%$'\r'}"
        trimmed="${line#"${line%%[![:space:]]*}"}"
        [[ -z "$trimmed" ]] && continue
        [[ "$trimmed" == \#* ]] && continue
        if [[ "$line" != *:* ]]; then
            echo "check-core-posix: malformed allowlist entry: $line" >&2
            exit 2
        fi
        AL_PATH+=("${line%%:*}")
        AL_REGEX+=("${line#*:}")
        allowlist_entries=$((allowlist_entries + 1))
    done < "$allowlist"
fi

is_allowed() {
    # is_allowed <relpath> <line content>
    local rel="$1" content="$2" i
    [[ ${#AL_PATH[@]} -gt 0 ]] || return 1
    for i in "${!AL_PATH[@]}"; do
        [[ "${AL_PATH[$i]}" == "$rel" ]] || continue
        if [[ "$content" =~ ${AL_REGEX[$i]} ]]; then
            return 0
        fi
    done
    return 1
}

hits_total=${#HIT_RULE[@]}
violations=0
allowlisted=0
keys=()
if [[ $hits_total -gt 0 ]]; then
    mapfile -t keys < <(printf '%s\n' "${!HIT_RULE[@]}" | LC_ALL=C sort)
fi

if [[ ${#keys[@]} -gt 0 ]]; then
    for key in "${keys[@]}"; do
    [[ -n "$key" ]] || continue
    rel="${key%:*}"
    line_no="${key##*:}"
    rule="${HIT_RULE[$key]}"
    content="${HIT_TEXT[$key]}"
    status="VIOLATION"
    if [[ "$mode" != "strict" ]] && is_allowed "$rel" "$content"; then
        status="allowed"
        allowlisted=$((allowlisted + 1))
    else
        violations=$((violations + 1))
    fi
    if [[ "$mode" == "list" ]]; then
        printf '%s:%s: %s [%s] (%s)\n' "$rel" "$line_no" "$content" "$rule" "$status"
    elif [[ "$status" == "VIOLATION" ]]; then
        printf '%s:%s: %s [%s]\n' "$rel" "$line_no" "$content" "$rule"
    fi
done
fi

case "$mode" in
    list)
        printf 'check-core-posix: %d hit(s) across %d file(s); allowlist entries: %d\n' \
            "$hits_total" "$scanned" "$allowlist_entries"
        exit 0
        ;;
    strict)
        printf 'check-core-posix: strict: %d hit(s) across %d file(s) (allowlist ignored)\n' \
            "$hits_total" "$scanned"
        [[ $violations -eq 0 ]] || exit 1
        exit 0
        ;;
    *)
        printf 'check-core-posix: %d hit(s) across %d file(s); %d violation(s), %d allowlisted\n' \
            "$hits_total" "$scanned" "$violations" "$allowlisted"
        [[ $violations -eq 0 ]] || exit 1
        exit 0
        ;;
esac
