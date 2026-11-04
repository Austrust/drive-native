import hashlib,json,re,sys
from pathlib import Path
root=Path(sys.argv[1]).resolve()
manifest=json.loads((root/'MANIFEST.json').read_text())
bad=[]
for name,expected in manifest['files'].items():
    path=root/name
    if not path.is_file():bad.append('missing: '+name);continue
    with path.open('rb') as stream:actual=hashlib.file_digest(stream,'sha256').hexdigest()
    if actual!=expected:bad.append('hash: '+name)
links=0
for name in ('README.md','REPORT.md','COVERAGE.md'):
    for target in re.findall(r'\[[^\]]*\]\(([^)]+)\)',(root/name).read_text()):
        if target.startswith(('http:','https:','#')):continue
        links+=1
        if not (root/target.split('#',1)[0]).exists():bad.append('link: '+name+' -> '+target)
print(json.dumps({'files_checked':len(manifest['files']),'links_checked':links,'errors':bad}))
assert not bad
