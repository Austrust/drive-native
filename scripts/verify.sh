#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p evidence
cargo test --workspace --all-features --locked 2>&1 | tee evidence/tests.log
bash scripts/check-core-posix.sh 2>&1 | tee evidence/core-posix.log
if [[ "${1:-}" == "--tests-only" ]]; then exit 0; fi
cargo build --release --locked -p drive-core --bin drive-native-lab
./target/release/drive-native-lab --simulate | tee evidence/simulation.json
