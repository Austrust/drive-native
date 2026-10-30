"""Ship readable documentation, linking unbundled evidence to the exact source."""
import argparse
import os
from pathlib import Path
import re
import subprocess
from urllib.parse import quote


def stage(project, destination):
    commit = os.environ.get('DRIVE_NATIVE_SOURCE_COMMIT', '')
    if not commit and (project / '.git').exists():
        if subprocess.check_output(['git', '-C', str(project), 'status', '--porcelain'], text=True).strip():
            raise ValueError('documentation identity requires a clean source checkout')
        commit = subprocess.check_output(['git', '-C', str(project), 'rev-parse', 'HEAD'], text=True).strip()
    if not re.fullmatch(r'[0-9a-f]{40}', commit):
        raise ValueError('set DRIVE_NATIVE_SOURCE_COMMIT to the source archive commit')
    version = subprocess.check_output(['dpkg-parsechangelog', '-S', 'Version'], cwd=project, text=True).strip()
    shipped = {
        'README.md', 'DELIVERY_GATES.md', 'docs/P0.md', 'docs/DURABILITY.md',
        'docs/REFERENCES.md', 'docs/PERFORMANCE.md', 'docs/RELEASE_NOTES.md',
    }
    for name in sorted(shipped):
        source = project / name

        def link(match):
            target = match.group(1)
            if target.startswith('#') or re.match(r'[a-zA-Z][a-zA-Z0-9+.-]*:', target):
                return match.group(0)
            path, separator, anchor = target.partition('#')
            resolved = (source.parent / path).resolve()
            if not resolved.is_relative_to(project):
                raise ValueError(f'documentation link outside source: {name}')
            if not resolved.exists():
                raise ValueError(f'missing documentation target: {name}: {path}')
            relative = resolved.relative_to(project).as_posix()
            if relative in shipped:
                return match.group(0)
            kind = 'tree' if resolved.is_dir() else 'blob'
            url = f'https://github.com/Austrust/drive-native/{kind}/{commit}/{quote(relative)}'
            if separator:
                url += '#' + anchor
            return '](' + url + ')'

        content = re.sub(r'\]\(([^)]+)\)', link, source.read_text(encoding='utf-8'))
        output = destination / name
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(content, encoding='utf-8')
        output.chmod(0o644)
    for name, content in [('SOURCE_COMMIT', commit), ('BUILD_VERSION', version)]:
        output = destination / name
        output.write_text(content + '\n', encoding='utf-8')
        output.chmod(0o644)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('destination', type=Path)
    args = parser.parse_args()
    stage(Path(__file__).resolve().parents[1], args.destination.resolve())
