"""Post-G4 build: reuse byte-proven .9 fixtures, rebuild candidate in a new target."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import time

BASE=Path('/home/drive-native-test/.local/state/drive-native-ultra-benchmark')
FIXTURE='crates/drive-fs/examples/fsearch_fixture.rs'
def sha(path):
    with path.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--archives',type=Path,required=True)
    parser.add_argument('--build-root',type=Path,required=True)
    parser.add_argument('--reuse-build',type=Path,required=True)
    parser.add_argument('--build-authorized',action='store_true')
    args=parser.parse_args()
    run=args.build_root.resolve();old=args.reuse_build.resolve()
    assert args.build_authorized and run.parent==BASE and old.parent==BASE and run.name.startswith('build-')
    assert not run.exists()
    metadata=json.loads((args.archives/'source.json').read_text())
    previous=json.loads((old/'build.json').read_text())
    assert previous['build_complete'] and metadata['baseline_commit']==previous['baseline_commit']
    assert metadata['baseline_archive_sha256']==previous['baseline_archive_sha256']
    for role in ('baseline','candidate'):
        assert sha(args.archives/(role+'.tar'))==metadata[role+'_archive_sha256']
    run.mkdir();(run/'candidate').mkdir();(run/'bin').mkdir();(run/'logs').mkdir()
    with tarfile.open(args.archives/'candidate.tar') as archive:archive.extractall(run/'candidate',filter='data')
    assert sha(run/'candidate'/FIXTURE)==previous['fixture_sha256'],'fixture changed: baseline must be rebuilt'
    assert sha(old/'baseline'/FIXTURE)==previous['fixture_sha256']
    # Prove the compiled baseline source still matches the original .9 archive,
    # except for the explicitly overlaid common fixture.
    verified=0
    with tarfile.open(args.archives/'baseline.tar') as archive:
        for member in archive.getmembers():
            if not member.isfile() or member.name==FIXTURE:continue
            stream=archive.extractfile(member)
            expected=hashlib.file_digest(stream,'sha256').hexdigest()
            assert sha(old/'baseline'/member.name)==expected,member.name
            verified+=1
    for role in ('baseline','baseline-diagnostic'):
        source=Path(previous['binaries'][role]['path'])
        assert sha(source)==previous['binaries'][role]['sha256']
        shutil.copy2(source,run/'bin'/role)
    toolchain=subprocess.check_output(['cargo','--version'])+subprocess.check_output(['rustc','--version'])
    (run/'logs/toolchain.log').write_bytes(toolchain)
    assert toolchain==(old/'logs/toolchain.log').read_bytes(),'toolchain differs: baseline must be rebuilt'
    for filename in ('baseline-ordinary.log','baseline-diagnostic.log'):
        shutil.copy2(old/'logs'/filename,run/'logs'/('reused-'+filename))
    shutil.copy2(args.archives/'source.json',run/'source.json')
    shutil.copytree(old/'target-candidate',run/'target-candidate',symlinks=True)
    before={str(p.relative_to(run/'candidate')):sha(p) for p in (run/'candidate').rglob('*') if p.is_file()}
    environment=os.environ.copy()
    for key in ('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','DRIVE_NATIVE_FS_DIAGNOSTICS','DRIVE_NATIVE_FS_INDEX','DRIVE_NATIVE_FS_TTL_MS'):
        environment.pop(key,None)
    environment.update(CARGO_TARGET_DIR=str(run/'target-candidate'),CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='2')
    commands={}
    for diagnostic in (False,True):
        role='candidate-diagnostic' if diagnostic else 'candidate'
        command=['cargo','rustc' if diagnostic else 'build','--release','--locked','--offline','-j2','-p','drive-fs','--example','fsearch_fixture']
        if diagnostic:command+=['--','--cfg','drive_fs_diagnostics']
        log=run/'logs'/('candidate-diagnostic.log' if diagnostic else 'candidate-ordinary.log')
        started=time.time()
        with log.open('w') as output:
            result=subprocess.run(command,cwd=run/'candidate',env=environment,stdout=output,stderr=subprocess.STDOUT)
        commands[role]={'command':command,'exit_code':result.returncode,'start_utc_unix':started,'end_utc_unix':time.time()}
        assert result.returncode==0,role
        shutil.copy2(run/'target-candidate/release/examples/fsearch_fixture',run/'bin'/role)
    changed=[name for name,expected in before.items() if sha(run/'candidate'/name)!=expected]
    assert not changed
    metadata.update(build_complete=True,executed='offline candidate builds with proven baseline ELF reuse; no tests or scans',
        built_at_utc_unix=time.time(),fixture_sha256=sha(run/'candidate'/FIXTURE),commands=commands,
        baseline_reuse={'previous_build_json':str(old/'build.json'),'previous_build_json_sha256':sha(old/'build.json'),
                       'baseline_product_files_verified_against_d555944_archive':verified,'fixture_bytes_identical':True,'toolchain_identical':True},
        binaries={role:{'path':str(run/'bin'/role),'sha256':sha(run/'bin'/role)} for role in ('baseline','candidate','baseline-diagnostic','candidate-diagnostic')},
        logs={p.name:sha(p) for p in (run/'logs').iterdir()},baseline_example_cfg='drive_fs_baseline',
        diagnostic_example_cfg='drive_fs_diagnostics',product_rustflags=None,candidate_changed_source_files=changed)
    (run/'build.json').write_text(json.dumps(metadata,indent=2)+'\n')
    print(json.dumps({'build_complete':True,'manifest':str(run/'build.json')}))

if __name__=='__main__':main()
