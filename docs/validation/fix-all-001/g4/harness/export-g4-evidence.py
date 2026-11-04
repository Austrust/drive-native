"""Copy closed verification evidence to a new workspace folder; retain originals."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

BASE = Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
WORK = Path('/mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/ultra-benchmark')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--destination', required=True, type=Path)
    parser.add_argument('--runs', nargs='+', required=True)
    args = parser.parse_args()
    destination = args.destination.resolve()
    assert destination.is_relative_to(WORK) and destination != WORK
    destination.mkdir(exist_ok=False)
    sources = []
    for name in args.runs:
        run = (BASE / name).resolve()
        assert run.parent == BASE
        target = destination / name
        target.mkdir()
        sources.append(str(run))
        if name.startswith('g4-'):
            assert json.loads((run / 'summary.json').read_text())['complete']
            for path in run.glob('*.json'):
                shutil.copy2(path, target / path.name)
            for folder in ('logs', 'screenshots', 'acl-screenshots'):
                if (run / folder).exists():
                    shutil.copytree(run / folder, target / folder)
            supplemental = run / 'supplement'
            if supplemental.exists():
                assert json.loads((supplemental / 'summary.json').read_text())['complete']
                (target / 'supplement').mkdir()
                for path in supplemental.iterdir():
                    if path.is_file():
                        shutil.copy2(path, target / 'supplement' / path.name)
                if (supplemental / 'mainwindow-screenshots').exists():
                    shutil.copytree(supplemental / 'mainwindow-screenshots', target / 'supplement/mainwindow-screenshots')
            private = run / 'nautilus-private'
            if private.exists():
                (target / 'nautilus-private').mkdir()
                for path in private.iterdir():
                    if path.is_file():
                        shutil.copy2(path, target / 'nautilus-private' / path.name)
                shutil.copytree(private / 'debs', target / 'nautilus-private/debs')
            for history in run.glob('history-scale*'):
                if not history.is_dir():continue
                (target / history.name).mkdir()
                for path in history.iterdir():
                    if path.is_file() and path.suffix in ('.json', '.log', '.py', '.png'):
                        shutil.copy2(path, target / history.name / path.name)
                shutil.copytree(history / 'probe', target / history.name / 'probe')
        elif name.startswith('schema-g4-'):
            for path in run.iterdir():
                if path.is_file():
                    shutil.copy2(path, target / path.name)
            shutil.copytree(run / 'logs', target / 'logs')
            for version in (23, 24, 25):
                probe = run / f'probe{version}'
                if probe.exists():
                    shutil.copytree(probe, target / probe.name)
        elif name.startswith('functional-g4-'):
            assert json.loads((run / 'result.json').read_text())['complete']
            # Keep index/export/truth and stderr, omit only synthetic fixture state/body trees.
            shutil.copy2(run / 'result.json', target / 'result.json')
            for case in ('main', 'other'):
                root = run / case
                out = target / case
                out.mkdir()
                for path in root.iterdir():
                    if path.name != 'fixture':
                        if path.is_dir():
                            shutil.copytree(path, out / path.name)
                        elif path.is_file():
                            shutil.copy2(path, out / path.name)
                fixture = root / 'fixture'
                (out / 'fixture').mkdir()
                for filename in ('truth.json',):
                    if (fixture / filename).exists():
                        shutil.copy2(fixture / filename, out / 'fixture' / filename)
        else:
            raise AssertionError('not a recognized private verification run')
    manifest = {'original_roots': sources, 'originals_retained': True, 'files': {}}
    for path in destination.rglob('*'):
        if path.is_file():
            with path.open('rb') as stream:
                manifest['files'][str(path.relative_to(destination))] = hashlib.file_digest(stream, 'sha256').hexdigest()
    (destination / 'evidence-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps({'destination': str(destination), 'files': len(manifest['files'])}))


if __name__ == '__main__':
    main()
