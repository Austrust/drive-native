#!/usr/bin/env python3
"""Patch the frozen 012 native-loopback harness for the intended T1 delta only.

Background: the 012 harness asserted the pre-T1 classification table from
docs/validation/issue7-native-001/COVERAGE.md. Commit fefb8ee intentionally
classifies a direct `upload` reply 0x9003 as sproto::Error::MutationPermissionDenied
(the only proven pre-commit ACL refusal). Because the native `upload` wire action
also carries mkdir, the sproto public API (Client::mkdir_native -> namespace_request
with action "upload") returns the same typed variant for mkdir 0x9003; the runtime
worker downgrades creation to a hint, but the wire-level matrix observes the typed
variant.

Edits: one classify arm, the two expectation changes, and four adversarial
additions proving the typed refusal does not leak into download or handshake
classification. Every other case keeps the 012 expectation byte-for-byte.
The script is deterministic and verifies each edit applies exactly once.
"""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
HARNESS = HERE / 'native-loopback-013'
MAIN = HARNESS / 'main.rs'
FROZEN = HERE / 'native-loopback' / 'main.rs'

EDITS = [
    (
        'classify arm',
        '        Error::MutationPermissionUncertain { code } => json!({"kind":"mutation_permission_uncertain", "code":code}),\n',
        '        Error::MutationPermissionUncertain { code } => json!({"kind":"mutation_permission_uncertain", "code":code}),\n'
        '        Error::MutationPermissionDenied { code } => json!({"kind":"mutation_permission_denied", "code":code}),\n',
    ),
    (
        'refusal expectation for upload/mkdir 0x9003',
        '        } else if (matches!(action, Action::Upload | Action::Mkdir | Action::Rename | Action::Remove) && matches!(code,0x3002 | 0x3004))\n'
        '            || (matches!(action,Action::Upload | Action::Mkdir) && matches!(code,0x9003 | 0x9004))\n',
        '        } else if matches!(action, Action::Upload | Action::Mkdir) && code == 0x9003 {\n'
        '            // T1 delta: the only proven pre-commit ACL refusal, typed at the\n'
        '            // direct upload reply boundary (creation/mkdir is typed at wire\n'
        '            // level but downgraded to a hint by the runtime worker).\n'
        '            ("mutation_permission_denied", Some(code))\n'
        '        } else if (matches!(action, Action::Upload | Action::Mkdir | Action::Rename | Action::Remove) && matches!(code,0x3002 | 0x3004))\n'
        '            || (matches!(action,Action::Upload | Action::Mkdir) && matches!(code,0x9004))\n',
    ),
    (
        'adversarial additions (download/setup must stay generic)',
        '        cases.push(refusal(Action::Download,code));\n'
        '    }\n',
        '        cases.push(refusal(Action::Download,code));\n'
        '    }\n'
        '    // T1 adversarial additions: the typed pre-commit upload refusal must\n'
        '    // not leak into download responses or handshake/connect classification.\n'
        '    for code in [0x9003u64, 0xa004] {\n'
        '        cases.push(refusal(Action::Download, code));\n'
        '    }\n'
        '    for stage in [Stage::Auth, Stage::Connect] {\n'
        '        cases.push(Case { name:format!("t1_setup_{stage:?}_9003_is_generic"), action:Action::Download, stage,\n'
        '            reply:json!({"error":{"code":0x9003u64,"reason":"synthetic setup refusal"}}),\n'
        '            expected:Expected::Error { kind:"server", code:Some(0x9003) } });\n'
        '    }\n',
    ),
]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    text = MAIN.read_text()
    report = {'frozen_012_main_sha256': sha(FROZEN), 'pre_patch_main_sha256': sha(MAIN)}
    for label, old, new in EDITS:
        count = text.count(old)
        if count != 1:
            raise SystemExit(f'edit {label!r} matched {count} times, expected 1')
        text = text.replace(old, new)
        report[label] = {'matched': count, 'applied': True}
    MAIN.write_text(text)
    report['post_patch_main_sha256'] = sha(MAIN)
    report['scope'] = ('classify arm + upload/mkdir 0x9003 expectation (intended T1 delta) + '
                       'download 0x9003/0xa004 and setup 0x9003 generic-classification additions; '
                       'all other 012 expectations identical')
    output = HERE / 'native-loopback-013-patch.json'
    output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()