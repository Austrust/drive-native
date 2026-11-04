"""Derive exact test inventories from preserved G4 logs; never alters originals."""
import argparse
import hashlib
import json
from pathlib import Path
import re


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def rust_tests(text):
    suite = 'unknown'
    rows = []
    for line in text.splitlines():
        if 'Running ' in line and '/deps/' in line:
            suite = re.search(r'/deps/([^/)]+)', line).group(1)
            suite = re.sub(r'-[a-f0-9]{16}$', '', suite)
        elif re.search(r'Doc-tests\s+', line):
            suite = 'doc:' + line.split('Doc-tests', 1)[1].strip()
        match = re.match(r'^test (.+) \.\.\. (ok|FAILED|ignored)(?:, (.*))?$', line)
        if match:
            rows.append({'suite': suite, 'name': match[1],
                         'status': match[2], 'reason': match[3]})
    return rows


def gui_tests(text):
    # D-Bus activation may interrupt the first verbose unittest line before 'ok'.
    starts = list(re.finditer(r'^(test_\S+) \(([^\n]+?)\) \.\.\. ', text, re.M))
    rows = []
    for index, match in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(text)
        body = text[match.end():end]
        outcome = re.search(r'(?:^|\n)(ok|FAIL|ERROR|skipped[^\n]*)(?:\n|$)', body)
        if outcome:
            value = outcome[1]
            status = 'ok' if value == 'ok' else 'ignored' if value.startswith('skipped') else 'FAILED'
        else:
            value = None
            status = 'unparsed'
        rows.append({'suite': match[2], 'name': match[1], 'status': status,
                     'reason': value if status in ('ignored', 'unparsed') else None})
    return rows


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'derived output must be new'
    source = json.loads((args.run / 'summary.json').read_text())
    result = {'candidate_commit': source['candidate_commit'],
              'source_summary_sha256': digest(args.run / 'summary.json'),
              'main_matrix_complete': source['complete'], 'commands': [],
              'note': 'Counts are executions per command; repeated modes are not additive unique coverage.'}
    unique = {}
    for command in source['commands']:
        if 'exit_code' not in command:
            continue
        logfile = args.run / command['log']
        text = logfile.read_text(errors='replace')
        rows = rust_tests(text) if command['kind'] == 'rust' else gui_tests(text) if command['kind'] == 'gui' else []
        totals = {status: sum(row['status'] == status for row in rows)
                  for status in ('ok', 'FAILED', 'ignored', 'unparsed')}
        expected = command['counts']
        reconciled = not expected.get('reported') or all(
            totals[actual] == expected.get(key, 0)
            for actual, key in [('ok', 'passed'), ('FAILED', 'failed'), ('ignored', 'ignored')])
        result['commands'].append({'name': command['name'], 'exit_code': command['exit_code'],
                                   'reported_counts': expected, 'inventory_counts': totals,
                                   'counts_reconciled': reconciled, 'log': command['log'],
                                   'log_sha256': digest(logfile), 'tests': rows})
        for row in rows:
            key = row['suite'] + '::' + row['name']
            record = unique.setdefault(key, {'suite': row['suite'], 'name': row['name'], 'observations': []})
            record['observations'].append({'command': command['name'], 'status': row['status'], 'reason': row['reason']})
    result['unique_tests'] = [unique[key] for key in sorted(unique)]
    result['all_reported_counts_reconciled'] = all(row['counts_reconciled'] for row in result['commands'])
    args.output.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'commands': len(result['commands']), 'unique_test_names': len(unique),
                      'all_reported_counts_reconciled': result['all_reported_counts_reconciled']}))


if __name__ == '__main__':
    main()
