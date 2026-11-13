#!/usr/bin/env bash
# Issue #94 / task-18 A/B performance orchestrator (local side).
#
# Runs interleaved baseline <-> fixed pairs of one `status_latency_210k` mode on
# the remote host under the exclusive heavy lock and prints medians of the key
# per-interface metrics per tree. Both trees MUST contain this branch's harness
# (`crates/drive-runtime/tests/status_latency_210k.rs`) and this script
# (`scripts/perf-gate-94.sh`) so the measurement code is identical; only the
# product code differs.
#
# Baseline convention (validation worktree = origin/main product):
#   BASE_SRC=src-val      BASE_TARGET=t-val    BASE_HEAD=origin/main sha
# Fixed tree (integrated product + the same harness overlay):
#   FIX_SRC=src-integ     FIX_TARGET=t-integ   FIX_HEAD=integration sha
# Fixtures are copied once (<fixture>-base -> <fixture>-integ) so both columns
# start from the same SQLite/blob fixture.
#
# Usage: scripts/perf-ab-94.sh <mode> [rounds]
#   mode: cold | warm | mixed | degraded
# Env overrides: HOST, ROUNDS, HARNESS_TEST_FILTER, plus BASE_*/FIX_* vars.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
HOST="${HOST:-workstation202611}"
REMOTE_HOME="${REMOTE_HOME:-/home/caozijian}"
MODE="${1:?usage: perf-ab-94.sh <cold|warm|mixed|degraded> [rounds]}"
ROUNDS="${2:-${ROUNDS:-3}}"

BASE_SRC="${BASE_SRC:-src-val}"
BASE_TARGET="${BASE_TARGET:-t-val}"
BASE_HEAD="${BASE_HEAD:-7d664e06d86547aff8365842fff03aad5a2cdb29}"
BASE_FIXTURE="${BASE_FIXTURE:-$REMOTE_HOME/work/r5-fixtures/lat210k-base}"
FIX_SRC="${FIX_SRC:-src-integ}"
FIX_TARGET="${FIX_TARGET:-t-integ}"
FIX_HEAD="${FIX_HEAD:?set FIX_HEAD to the integrated tree SHA}"
FIX_FIXTURE="${FIX_FIXTURE:-${BASE_FIXTURE%-base}-integ}"

# Logs live beside the repo worktree (workspace-level notes/, not committed).
NOTES_DIR="${NOTES_DIR:-$(cd "$ROOT/.." && pwd)/notes/test-logs/r5/val}"
LOCAL_LOG_DIR="$NOTES_DIR"
mkdir -p "$LOCAL_LOG_DIR"

mode_env=()
case "$MODE" in
    cold)
        mode_env=("LAT_SERVICE=") ;;
    warm)
        mode_env=("LAT_SERVICE=start" "LAT_WARM=1") ;;
    degraded)
        mode_env=("LAT_SERVICE=start") ;;
    mixed)
        mode_env=("LAT_SERVICE=start" "LAT_WARM=1") ;;
    *)
        echo "unknown mode $MODE" >&2; exit 2 ;;
esac

run_one() {
    local tree="$1" src="$2" target="$3" head="$4" fixture="$5" round="$6"
    local remote_log="$REMOTE_HOME/work/r5-evidence/val/ab-$MODE-$tree-r$round.log"
    local local_log="$LOCAL_LOG_DIR/ab-$MODE-$tree-r$round.log"
    echo "[ab-94] $tree round=$round src=$src target=$target head=$head" >&2
    local bin
    bin=$(ssh "$HOST" "ls -t $REMOTE_HOME/work/r5-targets/$target/release/deps/status_latency_210k-* 2>/dev/null | grep -v '\.d\$' | head -1")
    if [ -z "$bin" ]; then
        echo "[ab-94] missing release test binary for target $target (build it first)" >&2
        return 1
    fi
    # The mode variables must reach the test process: export them in the
    # remote shell; the gate script only overrides LAT_MODE/LAT_HEAD/LAT_ROUND.
    local exports=""
    for entry in "${mode_env[@]}"; do
        [ -n "$entry" ] && exports+="export $entry; "
    done
    ssh "$HOST" "set -o pipefail; $exports$REMOTE_HOME/work/dn-r5-run.sh heavy $target $src -- scripts/perf-gate-94.sh --mode $MODE --bin $bin --head $head --round $round --fixture $fixture --log $remote_log" >/dev/null 2>&1 || true
    scp -q "$HOST:$remote_log" "$local_log"
}

for round in $(seq 1 "$ROUNDS"); do
    if [ $((round % 2)) -eq 1 ]; then
        run_one base "$BASE_SRC" "$BASE_TARGET" "$BASE_HEAD" "$BASE_FIXTURE" "$round"
        run_one fixed "$FIX_SRC" "$FIX_TARGET" "$FIX_HEAD" "$FIX_FIXTURE" "$round"
    else
        run_one fixed "$FIX_SRC" "$FIX_TARGET" "$FIX_HEAD" "$FIX_FIXTURE" "$round"
        run_one base "$BASE_SRC" "$BASE_TARGET" "$BASE_HEAD" "$BASE_FIXTURE" "$round"
    fi
done

python3 - "$MODE" "$ROUNDS" "$LOCAL_LOG_DIR" <<'PY'
import json
import re
import statistics
import sys
from pathlib import Path

mode, rounds, log_dir = sys.argv[1], int(sys.argv[2]), Path(sys.argv[3])
interfaces = [
    "states_root", "dir2000", "overview", "fuse_dir2000",
    "states_conc16", "mixed:states", "mixed:dir", "mixed:overview",
    "classic:states", "classic:all", "mixed:all",
]
rows = {"base": [], "fixed": []}
for round_index in range(1, rounds + 1):
    for tree in ("base", "fixed"):
        path = log_dir / f"ab-{mode}-{tree}-r{round_index}.log"
        if not path.exists():
            continue
        text = path.read_text(errors="replace")
        entry = {"verdict": None, "checks_failed": [], "interfaces": {}, "peak_rss_kb": None,
                 "warm": None, "log": str(path.name)}
        match = re.search(r"^#94 VERDICT (\{.*\})$", text, re.MULTILINE)
        if match:
            try:
                verdict = json.loads(match.group(1))
                entry["verdict"] = verdict.get("verdict")
                entry["checks_failed"] = verdict.get("checks_failed", [])
                context = verdict.get("context", {})
                entry["peak_rss_kb"] = context.get("peak_rss_kb")
                entry["warm"] = context.get("projection")
            except json.JSONDecodeError:
                entry["verdict"] = "PARSE_ERROR"
        for line in text.splitlines():
            if not line.startswith("#94 PER_INTERFACE "):
                continue
            try:
                payload = json.loads(line[len("#94 PER_INTERFACE "):])
            except json.JSONDecodeError:
                continue
            entry["interfaces"][payload.get("interface")] = payload
        rows[tree].append(entry)

def median(tree, interface, field):
    values = [e["interfaces"][interface][field] for e in rows[tree]
              if interface in e["interfaces"] and e["interfaces"][interface].get(field) is not None]
    return statistics.median(values) if values else None

print(f"#94 A/B mode={mode} rounds={rounds}")
print(f"{'metric':28s} {'base':>12s} {'fixed':>12s}  {'delta':>10s}")
for interface in interfaces:
    for field in ("p95_ms", "p99_ms", "max_ms", "timeouts", "errors", "n"):
        base = median("base", interface, field)
        fixed = median("fixed", interface, field)
        if base is None or fixed is None:
            continue
        delta = fixed - base
        suffix = "%" if field.endswith("_ms") and base else ""
        if base and field.endswith("_ms"):
            delta_text = f"{delta:+.1f} ({100.0 * delta / base:+.1f}%)"
        else:
            delta_text = f"{delta:+.1f}"
        print(f"{interface + '.' + field:28s} {base:>12.3f} {fixed:>12.3f}  {delta_text:>10s}")
for tree in ("base", "fixed"):
    verdicts = [e["verdict"] or "MISSING" for e in rows[tree]]
    failed = sorted({check for e in rows[tree] for check in e["checks_failed"]})
    print(f"#94 A/B verdicts {tree}: {verdicts} failed_checks={failed}")
    warm = [e["warm"] for e in rows[tree] if e["warm"]]
    if warm:
        ready = [w.get("ready") for w in warm]
        ready_ms = [w.get("ready_ms") for w in warm if w.get("ready_ms") is not None]
        warm_p95 = [w.get("warm_p95_ms") for w in warm if w.get("warm_p95_ms") is not None]
        print(f"#94 A/B warm {tree}: ready={ready} ready_ms_median={statistics.median(ready_ms) if ready_ms else None} warm_p95_median={statistics.median(warm_p95) if warm_p95 else None}")
PY
