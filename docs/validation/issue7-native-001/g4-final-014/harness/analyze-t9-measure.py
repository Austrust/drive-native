#!/usr/bin/env python3
"""Independently re-derive the T9 real-NAS measurement claims from the committed
JSONL originals and check them against the report text."""
import argparse
import json
from pathlib import Path


def load(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--docs', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    docs = args.docs
    root_scopes = load(docs / 'measure-root-scopes-20261105.jsonl')
    home5 = load(docs / 'measure-home-attempts5-20261105.jsonl')
    home8 = load(docs / 'measure-home-attempts8-20261105.jsonl')
    enum = load(docs / 'measure-drive-enum-20261105.jsonl')

    def seed_row(rows, label, view):
        for row in rows:
            if row.get('stage') == 'seed' and row.get('label') == label and row.get('view_id') == view:
                return row
        raise SystemExit(f'seed row {label}/view {view} missing')

    root_mf = seed_row(root_scopes, 'ROOT_MF', 14)
    home_early = seed_row(root_scopes, 'HOME', 1)
    enum_row = next(row for row in root_scopes if row.get('stage') == 'enumeration')
    home5_row = seed_row(home5, 'HOME', 1)
    home8_row = seed_row(home8, 'HOME', 1)
    enum_big = next(row for row in enum if row.get('stage') == 'enumeration')

    def stats(row):
        phase = row['retry_phase']
        histogram = phase['attempt_histogram']
        derived_retries = sum((item['attempts'] - 1) * item['requests'] for item in histogram)
        derived_requests = sum(item['requests'] for item in histogram)
        return {'pages': row['pages'], 'events': row['events'],
                'elapsed_s': round(row['elapsed_s'], 3),
                'completed': row['completed'], 'disconnects': phase['disconnects'],
                'retries_field': phase['retries'], 'retries_derived': derived_retries,
                'requests_field': phase['requests'], 'requests_derived': derived_requests,
                'server_errors': phase['server_errors'],
                'attempt_histogram': histogram,
                'consistency': (derived_retries == phase['retries']
                                and derived_requests == phase['requests']
                                and derived_retries == phase['disconnects'])}
    root_mf_stats = stats(root_mf)
    home5_stats = stats(home5_row)
    home8_stats = stats(home8_row)
    report_claims = {
        'root_mf': {'pages': 76, 'events': 102014, 'elapsed_s': 215.0,
                    'disconnects': 25, 'retries': 25, 'server_errors': 0, 'completed': True},
        'home_attempts5': {'pages': 311, 'events': 148281, 'elapsed_s': 309.0,
                           'disconnects': 24, 'stop_reason': 'retry_exhausted'},
        'home_attempts8': {'pages': 289, 'events': 126129, 'elapsed_s': 422.7,
                           'disconnects': 7, 'stop_reason': 'deadline_or_page_cap'},
        'drive_enum': {'dirs': 5000, 'nodes': 20476, 'requests': 5001,
                       'elapsed_s': 159.1, 'disconnects': 0},
    }
    claims_ok = {
        'root_mf': (root_mf_stats['pages'] == 76 and root_mf_stats['events'] == 102014
                    and abs(root_mf_stats['elapsed_s'] - 215.0) < 1.0
                    and root_mf_stats['disconnects'] == 25
                    and root_mf_stats['retries_derived'] == 25
                    and root_mf_stats['server_errors'] == 0
                    and root_mf_stats['completed'] is True),
        'home_attempts5': (home5_stats['pages'] == 311 and home5_stats['events'] == 148281
                           and abs(home5_stats['elapsed_s'] - 309.0) < 1.0
                           and home5_stats['disconnects'] == 24
                           and home5_row['stop_reason'] == 'retry_exhausted'),
        'home_attempts8': (home8_stats['pages'] == 289 and home8_stats['events'] == 126129
                           and abs(home8_stats['elapsed_s'] - 422.7) < 1.0
                           and home8_stats['disconnects'] == 7
                           and home8_row['stop_reason'] == 'deadline_or_page_cap'),
        'drive_enum': (enum_big['dirs'] == 5000 and enum_big['nodes'] == 20476
                       and enum_big['stats']['requests'] == 5001
                       and abs(enum_big['elapsed_s'] - 159.1) < 1.0
                       and enum_big['stats']['disconnects'] == 0),
    }
    result = {
        'scope': 'independent re-derivation of T9 real-NAS measurement claims from committed JSONL; '
                 'not a new NAS measurement and not product behaviour evidence',
        'sources': {name: {'file': str(docs / name), 'lines': len(load(docs / name))}
                    for name in ('measure-root-scopes-20261105.jsonl',
                                 'measure-home-attempts5-20261105.jsonl',
                                 'measure-home-attempts8-20261105.jsonl',
                                 'measure-drive-enum-20261105.jsonl')},
        'root_mf': root_mf_stats, 'home_attempts5': home5_stats,
        'home_attempts8': home8_stats,
        'drive_enumeration': {'dirs': enum_big['dirs'], 'nodes': enum_big['nodes'],
                              'requests': enum_big['stats']['requests'],
                              'elapsed_s': round(enum_big['elapsed_s'], 3),
                              'disconnects': enum_big['stats']['disconnects']},
        'report_claims': report_claims,
        'claims_reproduced': claims_ok,
        'early_enumeration_row': {'dirs': enum_row['dirs'], 'nodes': enum_row['nodes']},
        'home_early_no_retry': home_early['no_retry_phase'],
        'passed': all(claims_ok.values()) and all(
            stats(row)['consistency'] for row in (root_mf, home5_row, home8_row)),
        'counts': {'passed': sum(1 for value in claims_ok.values() if value),
                   'failed': sum(1 for value in claims_ok.values() if not value),
                   'ignored': 0},
    }
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'passed': result['passed'], 'claims': claims_ok}))


if __name__ == '__main__':
    main()