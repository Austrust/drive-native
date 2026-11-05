import argparse,hashlib,json,pathlib,shutil,subprocess,tomllib
p=argparse.ArgumentParser();p.add_argument('--source',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True)
a=p.parse_args();assert not a.output.exists();(a.output/'src').mkdir(parents=True)
shutil.copy2(pathlib.Path(__file__).with_name('core-history-probe.rs'),a.output/'src/lib.rs')
(a.output/'Cargo.toml').write_text('''[package]
name="issue7-core-history-probe"
version="0.0.0"
edition="2024"
[dependencies]
drive-core={path="SOURCE/crates/drive-core"}
tempfile="3"
serde_json="1"
'''.replace('SOURCE',str(a.source)))
shutil.copy2(a.source/'Cargo.lock',a.output/'Cargo.lock')
proc=subprocess.run(['cargo','metadata','--offline','--format-version','1','--manifest-path',str(a.output/'Cargo.toml')],capture_output=True,text=True)
(a.output/'metadata.json').write_text(proc.stdout)
(a.output/'metadata.log').write_text(proc.stderr);assert proc.returncode==0,proc.stderr
old=tomllib.loads((a.source/'Cargo.lock').read_text())['package'];new=tomllib.loads((a.output/'Cargo.lock').read_text())['package']
identity=lambda p:(p['name'],p['version'],p.get('source'),p.get('checksum'))
registry=[p for p in new if p.get('source')];assert not [identity(p) for p in registry if identity(p) not in {identity(q) for q in old}]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
(a.output/'provenance.json').write_text(json.dumps({'source':str(a.source),'candidate_lock_sha256':sha(a.source/'Cargo.lock'),
 'harness_lock_sha256':sha(a.output/'Cargo.lock'),'harness_source_sha256':sha(a.output/'src/lib.rs'),
 'registry_dependencies_checked':len(registry),'registry_dependencies_match_frozen_candidate':True},indent=2)+'\n')
