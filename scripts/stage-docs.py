"""Ship readable documentation, linking unbundled evidence to the exact source."""
import argparse
import hashlib
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
    offline = os.environ.get('DRIVE_NATIVE_OFFLINE_DOCS') == '1'
    references = {}
    shipped = {
        'README.md', 'DELIVERY_GATES.md', 'docs/P0.md', 'docs/DURABILITY.md',
        'docs/REFERENCES.md', 'docs/PERFORMANCE.md', 'docs/RELEASE_NOTES.md',
        'docs/UBUNTU_INTEGRATION.md',
    }
    if offline:
        shipped.update({'docs/FIX_7_REPORT.md', 'docs/FIX_5_2_REPORT.md',
                        'docs/CONTRACTS.md', 'docs/UPGRADE_RECOVERY.md', 'docs/FILE_TIMESTAMPS.md'})
    for name in sorted(shipped):
        source = project / name

        def link(match):
            image, label, target = match.groups()
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
            if offline:
                entry = relative + ('#' + anchor if separator else '')
                reference = 'source-' + hashlib.sha256(entry.encode('utf-8')).hexdigest()[:16]
                references[entry] = reference
                local = os.path.relpath(destination / 'SOURCE_REFERENCE.md',
                                        (destination / name).parent).replace(os.sep, '/')
                # An unbundled image becomes a readable source reference, not
                # an attempted image load from an unavailable Markdown target.
                if image:
                    label = '源码图片：' + label
                return f'[{label}]({local}#{reference})'
            kind = 'tree' if resolved.is_dir() else 'blob'
            url = f'https://github.com/Austrust/drive-native/{kind}/{commit}/{quote(relative)}'
            if separator:
                url += '#' + anchor
            return f'{image}[{label}]({url})'

        content = re.sub(r'(!?)\[([^\]\n]*)\]\(([^)\n]+)\)', link, source.read_text(encoding='utf-8'))
        output = destination / name
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(content, encoding='utf-8')
        output.chmod(0o644)
    if offline:
        note = ["# 配套源码与未内置材料", "",
                f"包版本：`{version}`；冻结源码：`{commit}`。", "",
                "本地开发包没有发布到远端仓库。未随 deb 安装的材料在同一交付中的 "
                "`drive-native-source.tar.gz`；解压后，从 `source/` 目录按下列路径查找。"
                "这些路径是归档内位置，不表示已在线发布。", ""]
        for entry, reference in sorted(references.items()):
            note.extend([f'<a id="{reference}"></a>', f'- `{entry}`', ''])
        output = destination / 'SOURCE_REFERENCE.md'
        output.write_text('\n'.join(note), encoding='utf-8')
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
