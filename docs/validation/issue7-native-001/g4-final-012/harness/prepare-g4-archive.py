"""Prepare a new exact-commit archive and frozen harness metadata on Windows."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

p=argparse.ArgumentParser()
p.add_argument('--commit',required=True)
p.add_argument('--run',required=True)
p.add_argument('--authorization',required=True)
a=p.parse_args()
here=Path(__file__).resolve().parent
source=here.parent/'fix-7'
assert len(a.commit)==40 and all(c in '0123456789abcdef' for c in a.commit)
assert a.run.startswith('g4-') and a.run[3:].isdigit()
assert subprocess.check_output(['git','-C',str(source),'rev-parse',a.commit],text=True).strip()==a.commit
out=here/('prepared-'+a.run)
out.mkdir(exist_ok=False)
archive=out/'candidate.tar'
argv=['git','-C',str(source),'archive','--format=tar','--output='+str(archive),a.commit]
subprocess.run(argv,check=True)
def sha(path):
    with path.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
shutil.copy2(here/'run-g4.py',out/'run-g4.py')
record={'candidate_commit':a.commit,'candidate_archive_sha256':sha(archive),'archive_command':argv,
    'authorization':a.authorization,'harness_sha256':sha(out/'run-g4.py'),
    'cache_policy':'Cargo.lock registry dependencies only; no workspace/path artifacts; force clean four product packages; source metadata touched, bytes unchanged',
    'compile_cache_control':'CARGO_INCREMENTAL=0; profiles, parallelism and assertions unchanged',
    'host_space_gate':'C>=40GiB before new target; C>=10GiB before each command; unknown stops'}
(out/'source.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
print(json.dumps(record))
