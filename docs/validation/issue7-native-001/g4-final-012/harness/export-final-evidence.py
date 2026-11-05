"""Plan/export small immutable evidence only; never build, test or inspect target trees.

Default is a read-only plan. Use --export only after the coordinator closes every run.
The embedded 010-blocked bundle is verified against its own manifest and copied byte-for-byte.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
BASE = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
ALLOWED = {'.json', '.jsonl', '.log', '.py', '.sh', '.ps1', '.rs', '.toml', '.lock', '.md', '.png', '.sha256', '.txt', '.in', '.patch'}
FORBIDDEN_PARTS = {'target', 'source', 'cache', 'old-artifacts', 'fixtures', '__pycache__', 'registry', '.git'}
CHAIN_CASES = ['bridge_download_denied_3002', 'bridge_download_denied_3008',
    'bridge_profile_false_empty', 'bridge_profile_true_empty', 'bridge_profile_true_other_view',
    'bridge_late_read_after_profile', 'worker_rpc_generic_3004', 'worker_rpc_generic_8004',
    'worker_rpc_generic_9005', 'worker_rpc_generic_a005']

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def jbytes(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode()

def read(path):
    return json.loads(path.read_text())

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--candidate', required=True)
    parser.add_argument('--final', default='g4-012')
    parser.add_argument('--export', action='store_true')
    parser.add_argument('--allow-incomplete', action='store_true', help='archive a nonpassing status when all stage evidence exists; missing required inputs still stop export')
    args = parser.parse_args()
    assert re.fullmatch(r'g4-\d{3}', args.final) and args.final != 'g4-011'
    assert re.fullmatch(r'[0-9a-f]{40}', args.candidate)
    output = args.output.resolve()
    assert output.parent == HERE and not output.exists(), 'new direct child of harness directory required'
    spec = importlib.util.spec_from_file_location('frozen_inventory_parser', HERE / 'analyze-g4.py')
    inventory_parser = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(inventory_parser)
    files, generated, planned_hashes, prior, missing, run_rows, chain_rows = {}, {}, {}, [], [], [], []
    final_integrity = None

    def destination(value):
        path = PurePosixPath(str(value).replace('\\', '/'))
        assert not path.is_absolute() and '..' not in path.parts and path.parts, value
        assert not FORBIDDEN_PARTS.intersection(path.parts), value
        return str(path)

    def safe_file(path):
        assert path.is_file() and not path.is_symlink(), path
        assert path.suffix.lower() in ALLOWED and path.stat().st_size < 40 * 1024**2, path
        with path.open('rb') as stream:
            prefix = stream.read(4096)
        assert not prefix.startswith((b'\x7fELF', b'SQLite format 3\x00', b'!<arch>\n', b'MZ')), path
        assert not re.search(br'-----BEGIN (?:[A-Z ]+ )?PRIVATE KEY-----', prefix), path

    def add(path, relative, required=True):
        name = destination(relative)
        if not path.exists():
            missing.append({'path': str(path), 'destination': name, 'required': required})
            assert not required, path
            return
        safe_file(path)
        assert name not in files and name not in generated, name
        files[name], planned_hashes[name] = path, sha(path)

    def tree(path, relative):
        assert path.is_dir() and not path.is_symlink(), path
        # Only callers' explicitly named small evidence/harness directories are walked.
        for parent, dirs, names in os.walk(path, followlinks=False):
            folder = Path(parent)
            for child in dirs:
                assert not (folder / child).is_symlink(), folder / child
                assert child not in FORBIDDEN_PARTS, folder / child
            for name in names:
                item = folder / name
                add(item, PurePosixPath(relative) / item.relative_to(path).as_posix())

    def generate(name, value, raw=False):
        name = destination(name)
        assert name not in files and name not in generated, name
        generated[name] = value.encode() if raw else jbytes(value)

    def check(path, expected, authority):
        actual = sha(path) if path.is_file() and not path.is_symlink() else None
        prior.append({'path': str(path), 'authority': str(authority), 'expected_sha256': expected,
                      'actual_sha256': actual, 'matched': actual == expected})

    def local(root, value):
        candidate = Path(value)
        path = (candidate if candidate.is_absolute() else root / candidate).resolve()
        assert path.is_relative_to(root.resolve()), path
        return path

    def command_hashes(root, data, authority):
        for row in data.get('commands', []):
            if not isinstance(row, dict):
                continue  # Component lifecycle also records literal argv arrays.
            for field, hash_field in [('log', 'log_sha256'), ('stdout', 'stdout_sha256'),
                                     ('stderr', 'stderr_sha256'), ('result_path', 'result_sha256')]:
                if row.get(field) and row.get(hash_field):
                    check(local(root, row[field]), row[hash_field], authority)

    def derive(root, summary):
        result = {'candidate_commit': summary['candidate_commit'], 'source_summary_sha256': sha(root / 'summary.json'),
                  'main_matrix_complete': summary.get('complete'), 'commands': [],
                  'note': 'Per-command executions; repeated modes are not an additive unique total.'}
        unique = {}
        for command in summary.get('commands', []):
            if 'exit_code' not in command:
                continue
            log = local(root, command['log'])
            text = log.read_text(errors='replace')
            kind = command.get('kind')
            tests = inventory_parser.rust_tests(text) if kind == 'rust' else inventory_parser.gui_tests(text) if kind == 'gui' else []
            if kind == 'json' and command.get('result_path'):
                tests = [{'suite': command['name'], 'name': case['name'], 'status': 'ok' if case['passed'] else 'FAILED', 'reason': None}
                         for case in read(local(root, command['result_path']))['cases']]
            totals = {status: sum(row['status'] == status for row in tests) for status in ('ok', 'FAILED', 'ignored', 'unparsed')}
            expected = command.get('counts', {})
            reconciled = not expected.get('reported') or all(totals[key] == expected.get(name, 0)
                for key, name in [('ok', 'passed'), ('FAILED', 'failed'), ('ignored', 'ignored')])
            result['commands'].append({'name': command['name'], 'exit_code': command['exit_code'],
                'reported_counts': expected, 'inventory_counts': totals, 'counts_reconciled': reconciled,
                'log': command['log'], 'log_sha256': sha(log), 'tests': tests})
            for row in tests:
                key = row['suite'] + '::' + row['name']
                unique.setdefault(key, {'suite': row['suite'], 'name': row['name'], 'observations': []})['observations'].append(
                    {'command': command['name'], 'status': row['status'], 'reason': row['reason']})
        result['unique_tests'] = [unique[key] for key in sorted(unique)]
        result['all_reported_counts_reconciled'] = all(row['counts_reconciled'] for row in result['commands'])
        assert result['all_reported_counts_reconciled'], root
        return result

    # Embed the previous bundle unchanged; its old report retains its historical status.
    legacy = HERE / 'G4-evidence-bundle-010-blocked'
    legacy_manifest = read(legacy / 'MANIFEST.json')
    legacy_names = set(legacy_manifest['files']) | {'MANIFEST.json'}
    actual_names = {p.relative_to(legacy).as_posix() for p in legacy.rglob('*') if p.is_file()}
    assert actual_names == legacy_names, 'legacy bundle has unmanifested/missing files'
    for name in sorted(legacy_names):
        if name != 'MANIFEST.json':
            check(legacy / name, legacy_manifest['files'][name], legacy / 'MANIFEST.json')
        add(legacy / name, PurePosixPath('prior/G4-evidence-bundle-010-blocked') / name)

    for name in ('g4-011', args.final):
        root, rel = BASE / name, PurePosixPath('evidence') / name
        summary = read(root / 'summary.json')
        source = read(root / 'source.json')
        assert source['candidate_commit'] == summary['candidate_commit']
        assert summary['source'] == source
        if name == args.final:
            assert summary['candidate_commit'] == args.candidate
        command_hashes(root, summary, root / 'summary.json')
        for state, digest in summary.get('acl_screenshots', {}).items():
            if digest:
                check(root / 'acl-screenshots' / f'acl-{state}.png', digest, root / 'summary.json')
        for item in ('summary.json', 'source.json', 'source-hashes-before.json', 'final-integrity.json',
                     'cache-provenance.json', 'failure.txt', 'coordinator-stop.json', 'coordinator-control.json',
                     'native-loopback-results.json', 'native-loopback-elf.sha256', 'worker-versions.json',
                     'retry-preflight-clocks.json', 'retry-preflight-clock-audit.json',
                     'closeout.json', 'closeout-root-assisted.json'):
            add(root / item, rel / item, required=item in ('summary.json', 'source.json', 'source-hashes-before.json')
                or item == 'final-integrity.json' and name == args.final and not args.allow_incomplete)
        if name == args.final and (root / 'final-integrity.json').is_file():
            final_integrity = read(root / 'final-integrity.json')
            assert final_integrity['candidate_commit'] == args.candidate
        for folder in ('logs', 'acl-screenshots', 'screenshots'):
            if (root / folder).is_dir():
                tree(root / folder, rel / folder)
        parsed = derive(root, summary)
        existing = root / 'inventory-001.json'
        if existing.is_file():
            original = read(existing)
            check(root / 'summary.json', original['source_summary_sha256'], existing)
            assert original['all_reported_counts_reconciled']
            command_hashes(root, original, existing)
            assert original['commands'] == parsed['commands'], 'existing inventory differs from fresh parsing'
            add(existing, rel / 'inventory-001.json')
        else:
            generate(rel / 'inventory-001.json', parsed)
        for folder in ('native-loopback-crate', 'core-history-probe'):
            for item in ('Cargo.toml', 'Cargo.lock', 'provenance.json', 'lock-verification.json', 'metadata.log',
                         'metadata.json', 'cargo-metadata.json', 'src/main.rs', 'src/lib.rs'):
                if (root / folder / item).is_file():
                    add(root / folder / item, rel / folder / item)
        run_rows.append((name, summary, parsed))
        prepared = HERE / f'prepared-{name}'
        for item in ('source.json', 'run-g4.py', 'launcher.log'):
            add(prepared / item, PurePosixPath('harness') / prepared.name / item)
        assert read(prepared / 'source.json') == source
        check(prepared / 'run-g4.py', summary['harness_sha256'], root / 'summary.json')

        build_root = root / 'native-worker-chain-build-001'
        build = read(build_root / 'build.json')
        assert build['candidate_commit'] == summary['candidate_commit']
        command_hashes(build_root, build, build_root / 'build.json')
        driver = HERE / ('native-worker-chain' if name == 'g4-011' else 'native-worker-chain-012')
        for item, digest in build.get('driver_files', {}).items():
            check(local(driver, item), digest, build_root / 'build.json')
        for path in sorted(build_root.iterdir()):
            if path.is_file():
                add(path, rel / build_root.name / path.name)
        crate = build_root / 'native-worker-chain-crate'
        for item in ('Cargo.toml', 'Cargo.lock', 'provenance.json', 'lock-verification.json', 'src/main.rs', 'src/codec.rs'):
            add(crate / item, rel / build_root.name / crate.name / item)
        provenance, lock = read(crate / 'provenance.json'), read(crate / 'lock-verification.json')
        assert provenance['candidate_commit'] == summary['candidate_commit']
        assert lock['all_registry_dependencies_match_frozen_candidate']
        for item, digest in provenance['files'].items():
            # Certificates/private keys remain excluded; only included crate files are reread.
            if item != 'Cargo.lock' and str(rel / build_root.name / crate.name / item) in files:
                check(local(crate, item), digest, crate / 'provenance.json')
        check(crate / 'Cargo.lock', lock['harness_cargo_lock_sha256'], crate / 'lock-verification.json')
        check(build_root / 'metadata.stdout.log', lock['cargo_metadata_sha256'], crate / 'lock-verification.json')
        frozen_lock_hash = read(root / 'source-hashes-before.json')['Cargo.lock']
        assert provenance['source_cargo_lock_sha256'] == lock['source_cargo_lock_sha256'] == frozen_lock_hash
        assert build['source_lock_before'] == build['source_lock_after'] == frozen_lock_hash
        if name == args.final:
            execution_manifest = driver / 'HARNESS_EXECUTION_SHA256.json'
            for item, digest in read(execution_manifest)['files'].items():
                check(local(driver, item), digest, execution_manifest)
            reuse = build['completed_same_candidate_target_reuse']
            assert reuse and reuse['run'] == str(root)
            check(root / 'summary.json', reuse['summary_sha256'], build_root / 'build.json')
            check(root / 'supplemental-001/executed-elf-before.json', reuse['elf_witness_sha256'], build_root / 'build.json')
        for folder in (('native-worker-chain-denials-001', 'native-worker-chain-other-001') if name == 'g4-011' else ('native-worker-chain-001',)):
            directory = root / folder
            data = read(directory / 'results.json')
            assert data['candidate_commit'] == summary['candidate_commit']
            if name == args.final:
                assert set(data['runner_driver_files']) == {'run.py', 'space.py'}
            for item, digest in data.get('runner_driver_files', {}).items():
                check(local(driver, item), digest, directory / 'results.json')
            add(directory / 'results.json', rel / folder / 'results.json')
            names = [case['name'] for case in data['cases']]
            assert len(names) == len(set(names)) and set(names).issubset(CHAIN_CASES)
            for case in data['cases']:
                case_dir = directory / case['name']
                for item, field in [('output.log', 'log_sha256'), ('case.json', 'case_json_sha256')]:
                    if case.get(field):
                        check(case_dir / item, case[field], directory / 'results.json')
                    add(case_dir / item, rel / folder / case['name'] / item, required=not case.get('not_run'))
            measured = {'passed': sum(bool(c['passed']) for c in data['cases']),
                        'failed': sum(not c['passed'] and c['status'] != 'blocked' for c in data['cases']),
                        'ignored': sum(c['status'] == 'blocked' for c in data['cases']), 'total': len(data['cases'])}
            assert measured == data['counts']
            for item, digest in data['harness']['source_files'].items():
                check(local(crate, item), digest, directory / 'results.json')
            chain_rows.append((name, folder, data))

    final_root, final_rel = BASE / args.final, PurePosixPath('evidence') / args.final
    supplement_root = final_root / 'supplemental-001'
    supplement = read(supplement_root / 'summary.json')
    assert supplement['candidate_commit'] == args.candidate
    check(final_root / 'summary.json', supplement['main_summary_sha256'], supplement_root / 'summary.json')
    check(HERE / 'run-g4-012-supplements.py', supplement['harness_sha256'], supplement_root / 'summary.json')
    command_hashes(supplement_root, supplement, supplement_root / 'summary.json')
    tree(supplement_root, final_rel / supplement_root.name)
    generate(final_rel / supplement_root.name / 'derived-inventory.json', derive(supplement_root, supplement))
    component_root = final_root / 'component-policy-handover-001'
    component = read(component_root / 'result.json')
    assert component['new_commit'] == args.candidate
    command_hashes(component_root, component, component_root / 'result.json')
    add(component_root / 'result.json', final_rel / component_root.name / 'result.json')
    if (component_root / 'logs').exists():
        tree(component_root / 'logs', final_rel / component_root.name / 'logs')
    if (component_root / 'failure.txt').exists():
        add(component_root / 'failure.txt', final_rel / component_root.name / 'failure.txt')

    # Coordinator scripts are current navigation/driver evidence; frozen runners remain separate.
    for path in sorted(HERE.iterdir()):
        if path.is_file() and path.suffix in ('.py', '.sh', '.ps1', '.rs'):
            add(path, PurePosixPath('harness') / path.name)
    for folder in ('native-loopback', 'native-worker-chain', 'native-worker-chain-012'):
        tree(HERE / folder, PurePosixPath('harness') / folder)
    for folder in ('compact-001', 'compact-002'):
        tree(HERE / folder, PurePosixPath('environment') / folder)
    for path in sorted(HERE.iterdir()):
        if path.is_file() and (path.name.endswith('-cleanup.json') or path.name.startswith('cache-inventory-') and path.suffix == '.json'):
            add(path, PurePosixPath('environment') / path.name)
    inspection = HERE / 'resumed-011-readonly-inspection.jsonl'
    add(inspection, 'environment/resumed-011-readonly-inspection.jsonl')
    for line in inspection.read_text(encoding='utf-8-sig').splitlines()[:2]:
        observation = json.loads(line)
        observed_path = local(BASE / 'g4-011', observation['path'])
        assert observed_path.name == 'results.json'
        check(observed_path, observation['sha256'], inspection)

    old_cases = [case for name, _, data in chain_rows if name == 'g4-011' for case in data['cases']]
    assert len(old_cases) == 10 and {case['name'] for case in old_cases} == set(CHAIN_CASES)
    old_counts = {'passed': sum(bool(c['passed']) for c in old_cases), 'failed': sum(not c['passed'] for c in old_cases)}
    assert old_counts == {'passed': 8, 'failed': 2}
    assert {c['name'] for c in old_cases if not c['passed']} == set(CHAIN_CASES[:2])
    final_summary = next(summary for name, summary, _ in run_rows if name == args.final)
    final_chain = next(data for name, _, data in chain_rows if name == args.final)
    accepted = bool(final_summary.get('complete') and final_summary.get('all_commands_passed')
        and not final_summary.get('changed_source_files') and not final_summary.get('residual_owned_mounts')
        and final_integrity and final_integrity.get('passed')
        and supplement.get('complete') and supplement.get('all_passed') and component.get('passed')
        and final_chain.get('complete') and final_chain.get('all_passed')
        and {c['name'] for c in final_chain['cases']} == set(CHAIN_CASES))
    closeout = read(final_root / 'closeout-root-assisted.json')
    assert closeout['passed'] and closeout['candidate_commit'] == args.candidate
    check(final_root / 'closeout.json', closeout['prior_audit_sha256'], final_root / 'closeout-root-assisted.json')
    assert accepted or args.allow_incomplete, 'final evidence is not completely passing; use --allow-incomplete only to archive that status explicitly'
    mismatches = [row for row in prior if not row['matched']]
    assert not mismatches, json.dumps(mismatches, ensure_ascii=False)
    generate('PRIOR_HASH_AUDIT.json', {'checks': prior, 'matched': len(prior), 'mismatches': mismatches,
        'limitation': 'Only recorded historical hashes establish historical equality. Other files have export-time copy/manifest integrity only. Excluded ELF/source/DB contents are not reread.'})
    generate('PRIVATE_ORIGINALS.json', {'runs': [str(BASE / 'g4-011'), str(final_root)],
        'exclusions': ['target and source trees', 'Git tar archives', 'ELF/test binaries/libraries', 'databases', 'TLS certificate/private key files', 'package/cache trees'],
        'note': 'No excluded artifact is opened or rehashed. Its pre-existing metadata remains in the copied original summaries.'})
    generate('STATUS.json', {'candidate_commit': args.candidate, 'final_run': args.final,
        'acceptance_status': 'passed_matrix_and_supplements' if accepted else 'incomplete_or_failed',
        'old_413_tls_counterexample': old_counts, 'missing_optional_files': missing,
        'scope': 'G4 increment and synthetic private component/TLS validation; no new G5, real NAS or Debian lifecycle claim'})

    report = ['# #7 最终增量验证证据', '',
        f"候选 `{args.candidate}`；主矩阵与补测验收状态：`{'通过' if accepted else '未完成或失败'}`。原件独立保留，重复执行不相加为唯一测试总数。", '',
        '[旧 G4-006–010 包](prior/G4-evidence-bundle-010-blocked/REPORT.md)按原 MANIFEST 逐文件验证后原样嵌入；其 blocked 状态是历史记录，不重写。', '',
        '旧 `41365a6` 的真实本机 TLS→worker ELF→IPC→Bridge/RPC 十项补测为 **8 通过 / 2 失败**；失败是 3002/3008 拒绝后重试门禁的真实反例，后续修复通过不能覆盖它。六项 Bridge 链与四项直接 Worker RPC 的范围分别列出。', '',
        '## 主矩阵与补测', '']
    for name, data, _ in run_rows:
        rel = f'evidence/{name}'
        report += [f'### {name}', '', f"候选 `{data['candidate_commit']}`；complete=`{data.get('complete')}`，all_commands_passed=`{data.get('all_commands_passed')}`。",
            f'[summary]({rel}/summary.json) · [逐名 inventory]({rel}/inventory-001.json)', '',
            '| 命令 | 通过 | 失败 | 忽略 | 状态 |', '|---|---:|---:|---:|---|']
        for row in data.get('commands', []):
            counts = row.get('counts', {})
            values = [str(counts.get(key, 0)) if counts.get('reported') else '—' for key in ('passed', 'failed', 'ignored')]
            status = '未运行' if row.get('not_run') else '通过' if row.get('passed') else '失败/中断，见原件'
            label = f"[{row['name']}]({rel}/{row['log']})" if row.get('log') and f"{rel}/{row['log']}" in files else row['name']
            report.append('| ' + ' | '.join([label, *values, status]) + ' |')
        report += ['']
    for name, folder, data in chain_rows:
        report += [f"### {name}/{folder}", '', f"[results](evidence/{name}/{folder}/results.json)：`{data['counts']}`。", '',
            '| case | 路径范围 | 结果 |', '|---|---|---|']
        for case in data['cases']:
            scope = '实际 worker→IPC→Bridge' if case['name'].startswith('bridge_') else '直接 Worker RPC，未应用 Bridge'
            rel = f"evidence/{name}/{folder}/{case['name']}/case.json"
            label = f"[{case['name']}]({rel})" if rel in files else case['name']
            report.append(f"| {label} | {scope} | {case['status']} / passed={case['passed']} |")
        report += ['']
    report += [f"[容量及生命周期补测 summary](evidence/{args.final}/supplemental-001/summary.json) · [逐名容量 inventory](evidence/{args.final}/supplemental-001/derived-inventory.json) · [组件更换结果](evidence/{args.final}/component-policy-handover-001/result.json)", '',
        '210k ignored 用例属于功能容量验收，不是性能基准；只按本轮实际日志计数。组件更换为私有目录中实际 fsd/worker 配对，不是 Debian 安装、升级或重装。', '',
        '## 完整性、恢复与范围', '',
        '[历史 SHA 交叉核对](PRIOR_HASH_AUDIT.json)覆盖已有 summary→log/JSON、inventory→summary/log、TLS case/build 流与旧包 MANIFEST。没有先验 SHA 的文件只声明本次复制完整性。', '',
        'ELF witness 比较的是主矩阵结束时仍存在的 ELF 在补测前后的身份，不是每次执行瞬间采样，也不是 ELF 字节备份。源代码/ELF/数据库不导出；已有身份摘要随原件保存。', '',
        f'[普通用户结束审计](evidence/{args.final}/closeout.json)因两个 systemd 进程的 exe 链接权限限制记录为不完整；[独立 root 只读补核](evidence/{args.final}/closeout-root-assisted.json)确认无验证 ELF/cargo/rustc 进程、无残留挂载、原件未变。两份原件均保留，未把权限限制记为产品测试失败。', '',
        'harness 内旧 HARNESS_SHA256.json 是原静态快照，可能早于 wrapper 修订；不把它当作后续执行版本的权威。实际已有来源关联按 build.json.driver_files、生成 crate 的 provenance/lock-verification、TLS results 校验；其余文件只证明导出时字节身份。', '',
        '[compact-001 前](environment/compact-001/before.json) / [后](environment/compact-001/after.json)；[compact-002 前](environment/compact-002/before.json) / [后](environment/compact-002/after.json)。缓存盘点仅是对应时点的 guest 文件元数据，不能直接等同 Windows C 可回收量；清理/压缩的实际执行只以环境原件为准。', '',
        '[覆盖映射](COVERAGE.md) · [状态与缺项](STATUS.json) · [私有原件边界](PRIVATE_ORIGINALS.json) · [逐文件 SHA](MANIFEST.json)', '',
        '全部新链路均为合成本机服务与测试身份。真实 NAS ACL、空间能力来源、服务端部分提交真值、真实桌面面板及 Debian 包生命周期未实测。本包不新增 G5 性能结论；既有 G5 原件和未达标结论由对应报告维护。', '']
    generate('REPORT.md', '\n'.join(report), raw=True)
    coverage = ['# 覆盖映射', '',
        '[历史组件级映射](prior/G4-evidence-bundle-010-blocked/COVERAGE.md)与[本机 TLS 设计说明](harness/native-worker-chain-012/RUNBOOK.md)分别保留。下面只列入口和可证明的边界；实际结果见 REPORT/原始 JSON。', '',
        '| 用例组 | 实际入口 | 边界 |', '|---|---|---|',
        '| 3002/3008 拒绝 | bridge_download_denied_3002 / 3008 | 实际 TLS 服务→worker ELF→IPC→Bridge ensure；只污染目标 Read；再次 ensure 和到期后台刷新不发新 Download |',
        '| profile 三态 | bridge_profile_false_empty / true_empty / true_other_view | 实际 long-poll 响应穿过 worker 与 Bridge；profile-only 不依赖文件事件 |',
        '| 迟到 Read | bridge_late_read_after_profile | 双通道有界门闩；profile 失效先于旧 Download 拒绝发布 |',
        '| generic 四码 | worker_rpc_generic_3004 / 8004 / 9005 / a005 | 实际 worker RPC 分类；不称完整 Bridge 路径，不称 mutation 成功或无部分提交 |',
        '| 210k 容量 | default_budget_admits_210k_node_snapshot_without_holding_store | ignored 功能用例单独执行；不替代 G5 墙钟/每目录门槛 |',
        '| 私有组件更换 | component-policy-handover.py | 旧/新 fsd+worker、私有 D-Bus/FUSE、持久化策略/历史；不是 deb 生命周期 |', '',
        '主矩阵的 precise suite/name、INDEX0 N/A、ignored/doc 状态与重复执行观测均在逐轮 inventory；不把跨模式或跨候选计数累加成唯一总数。', '']
    generate('COVERAGE.md', '\n'.join(coverage), raw=True)

    names = set(files) | set(generated) | {'MANIFEST.json'}
    for name in ('REPORT.md', 'COVERAGE.md'):
        for link in re.findall(r'\]\(([^)]+)\)', generated[name].decode()):
            assert link in names, (name, link)
    estimate = sum(path.stat().st_size for path in files.values()) + sum(map(len, generated.values()))
    upper = estimate + sum(len(name.encode()) + 256 for name in names) + 4096
    print(json.dumps({'planned_files': len(names), 'estimated_upper_bytes': upper, 'under_100MiB': upper < 100 * 1024**2,
                      'prior_hash_checks': len(prior), 'acceptance_status': accepted, 'export_requested': args.export}), flush=True)
    if not args.export:
        return
    assert upper < 100 * 1024**2, 'report size before exporting above cap'
    assert shutil.disk_usage(HERE).free > upper + 32 * 1024**2
    output.mkdir()
    for name, path in files.items():
        assert sha(path) == planned_hashes[name], 'input changed since planning: ' + name
        dest = output / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, dest)
        assert sha(dest) == planned_hashes[name], 'copy mismatch: ' + name
    for name, content in generated.items():
        dest = output / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(content)
    manifest = {'candidate_commit': args.candidate, 'final_run': args.final,
                'acceptance_status': 'passed_matrix_and_supplements' if accepted else 'incomplete_or_failed',
                'files': {name: sha(output / name) for name in sorted(set(files) | set(generated))}}
    (output / 'MANIFEST.json').write_bytes(jbytes(manifest))
    print(json.dumps({'output': str(output), 'files': len(manifest['files']),
                      'manifest_sha256': sha(output / 'MANIFEST.json')}), flush=True)

if __name__ == '__main__':
    main()
