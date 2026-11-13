#!/usr/bin/env bash
# Issue #94 performance-gate runner (remote side).
#
# Runs one `status_latency_210k` measurement mode against a prebuilt test
# binary, requires the machine-readable `#94 VERDICT` line, records the report
# header fields (head/mode/round/loadavg) and exits with the gate verdict's
# exit code. The caller MUST hold the exclusive heavy lock, e.g.:
#
#   ~/work/dn-r5-run.sh heavy t-val src-val -- \
#     scripts/perf-gate-94.sh --mode warm --bin "$BIN" --head "$SHA" \
#       --round 1 --log ~/work/r5-evidence/val/gate-warm-r1.log
#
# Exit status: 0 = PASS, 1 = FAIL, 2 = INCONCLUSIVE, 3 = harness/infra error
# (missing binary/verdict line). `--expect-fail` inverts the check so an
# injected-failure run can be asserted in CI: it exits 0 only when the gate
# does NOT pass.
set -euo pipefail

usage() {
    cat >&2 <<'EOF'
usage: perf-gate-94.sh --mode <cold|warm|degraded|mixed> --bin <test-binary> --log <log-path>
                       [--head <sha>] [--round <n>] [--fixture <dir>]
                       [--inject <classic_timeout|interface_overrun|states_overrun>]
                       [--expect-fail] [--env K=V ...]
EOF
    exit 3
}

mode=""
bin=""
log=""
head="${LAT_HEAD:-unset}"
round="0"
fixture="${LAT_FIXTURE_DIR:-}"
inject=""
expect_fail=0
extra_env=()

while [ $# -gt 0 ]; do
    case "$1" in
        --mode) mode="${2:-}"; shift 2 ;;
        --bin) bin="${2:-}"; shift 2 ;;
        --log) log="${2:-}"; shift 2 ;;
        --head) head="${2:-}"; shift 2 ;;
        --round) round="${2:-}"; shift 2 ;;
        --fixture) fixture="${2:-}"; shift 2 ;;
        --inject) inject="${2:-}"; shift 2 ;;
        --expect-fail) expect_fail=1; shift ;;
        --env) extra_env+=("${2:-}"); shift 2 ;;
        *) usage ;;
    esac
done
[ -n "$mode" ] && [ -n "$bin" ] && [ -n "$log" ] || usage
[ -x "$bin" ] || { echo "[gate-94] missing test binary: $bin" >&2; exit 3; }
mkdir -p "$(dirname "$log")"

run_env=(
    "LAT_MODE=$mode"
    "LAT_HEAD=$head"
    "LAT_ROUND=$round"
)
if [ -n "$fixture" ]; then
    run_env+=("LAT_FIXTURE_DIR=$fixture")
fi
if [ -n "$inject" ]; then
    run_env+=("LAT_GATE_INJECT=$inject")
fi
run_env+=("${extra_env[@]}")

{
    echo "# gate-94 run"
    echo "# date=$(date -Iseconds) host=$(hostname)"
    echo "# head=$head mode=$mode round=$round inject=${inject:-none} expect_fail=$expect_fail"
    echo "# bin=$bin"
    echo "# fixture=${fixture:-<temporary>}"
    echo "# nproc=$(nproc) memory=$(awk '/MemTotal/ {printf "%.1fGiB", $2/1048576}' /proc/meminfo)"
    echo "# loadavg_before=$(cut -d' ' -f1-3 /proc/loadavg)"
    echo "# env: ${run_env[*]}"
} | tee "$log"

set +e
env "${run_env[@]}" python3 "$HOME/work/run-measured.py" "$bin" \
    --ignored --nocapture status_latency_210k 2>&1 | tee -a "$log"
harness_status=$?
set -e
echo "# loadavg_after=$(cut -d' ' -f1-3 /proc/loadavg)" | tee -a "$log"

verdict_line=$(grep -E '^#94 VERDICT ' "$log" | tail -n 1 || true)
if [ -z "$verdict_line" ]; then
    echo "[gate-94] no #94 VERDICT line in $log (harness_status=$harness_status)" >&2
    echo "# gate-94 result=infra_error (missing machine-readable verdict)" | tee -a "$log"
    exit 3
fi
verdict_json="${verdict_line#\#94 VERDICT }"
read -r verdict exit_code < <(python3 - "$verdict_json" <<'PY'
import json, sys
data = json.loads(sys.argv[1])
print(data.get("verdict", "UNKNOWN"), data.get("exit_code", 3))
PY
)
echo "# gate-94 verdict=$verdict declared_exit_code=$exit_code harness_status=$harness_status" | tee -a "$log"

if [ "$expect_fail" -eq 1 ]; then
    if [ "$verdict" = "PASS" ]; then
        echo "[gate-94] EXPECTATION VIOLATED: injected failure run reported PASS" >&2
        echo "# gate-94 result=expectation_violated" | tee -a "$log"
        exit 1
    fi
    echo "# gate-94 result=expected_non_pass (verdict=$verdict)" | tee -a "$log"
    exit 0
fi

case "$verdict" in
    PASS)
        echo "# gate-94 result=pass" | tee -a "$log"
        exit 0
        ;;
    FAIL)
        echo "# gate-94 result=fail" | tee -a "$log"
        exit 1
        ;;
    *)
        echo "# gate-94 result=inconclusive" | tee -a "$log"
        exit 2
        ;;
esac
