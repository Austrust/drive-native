"""Compact correctness-only view; original functional timings are not G5 data."""
import hashlib
import json
from pathlib import Path
import sys

path = Path(sys.argv[1])
data = json.loads(path.read_text())
checks = data['checks']
result = {'candidate_commit': data['candidate_commit'], 'complete': data['complete'],
          'performance_evidence': False, 'source_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
          'binaries': data['binaries'], 'checks': {}, 'normal_stops': data['normal_stops']}
for key in ('parallel_mount_not_blocked', 'cancelled_active_original_scan', 'mount_after_cancel',
            'local_edits', 'releasedir_cycles', 'hydrations'):
    if key in checks:
        result['checks'][key] = {field: value for field, value in checks[key].items() if 'seconds' not in field}
for key in ('restart_full_scan', 'local_changes_full_scan'):
    if key in checks:
        result['checks'][key] = {field: checks[key].get(field) for field in
            ('fresh_index_save_verified', 'indexed_count', 'missing', 'extra', 'duplicates',
             'invalid', 'attribute_mismatches', 'hydration_delta', 'index_sha256', 'export_sha256')}
result['index_and_ttl_diagnostics'] = {}
for role, sample in checks.get('final_diagnostics', {}).items():
    result['index_and_ttl_diagnostics'][role] = {row['stage']: row['count'] for row in sample['diagnostics']
        if row['stage'] in ('IndexHits', 'IndexMisses', 'IndexFallback', 'IndexDegraded', 'EntryTtlNonZero', 'EntryTtlZero')}
print(json.dumps(result, indent=2))
