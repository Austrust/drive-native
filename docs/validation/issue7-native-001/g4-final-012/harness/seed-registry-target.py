"""Create a new private target using only Cargo.lock registry dependencies."""
import argparse,hashlib,importlib.util,json,pathlib,shutil,tomllib
p=argparse.ArgumentParser()
p.add_argument('--source',type=pathlib.Path,required=True)
p.add_argument('--seed',type=pathlib.Path,required=True)
p.add_argument('--target',type=pathlib.Path,required=True)
p.add_argument('--output',type=pathlib.Path,required=True)
p.add_argument('--host-space-helper',type=pathlib.Path,required=True)
a=p.parse_args();base=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
source,seed,target,output=(path.resolve() for path in (a.source,a.seed,a.target,a.output))
assert all(path.is_relative_to(base) for path in (source,seed,target,output))
assert not target.exists() and not output.exists() and not target.is_relative_to(source)
spec=importlib.util.spec_from_file_location('frozen_host_gate',a.host_space_helper)
helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
observation=helper.host_c_space()
result={'source':str(source),'seed':str(seed),'target':str(target),'complete':False,
    'host_space_before':observation,'minimum_host_bytes':40*1024**3,
    'host_space_helper_sha256':hashlib.sha256(a.host_space_helper.read_bytes()).hexdigest(),
    'selection':'Only registry+ package names from frozen Cargo.lock; debug deps/fingerprint/build. No path products, test ELF or incremental cache.'}
output.parent.mkdir(parents=True,exist_ok=True)
def save():output.write_text(json.dumps(result,indent=2)+'\n')
save();assert observation['free_bytes'] is not None and observation['free_bytes']>=40*1024**3,'host space gate'
packages=tomllib.loads((source/'Cargo.lock').read_text())['package']
names={row['name'] for row in packages if row.get('source','').startswith('registry+')}
normalized={name.replace('-','_') for name in names}
files=total=0
try:
    for folder in ('deps','.fingerprint','build'):
        origin=seed/'debug'/folder;dest=target/'debug'/folder;dest.mkdir(parents=True)
        for item in origin.iterdir():
            if folder=='deps':
                if not item.is_file() or item.suffix not in ('.rlib','.rmeta','.so','.d'):continue
                if item.stem.removeprefix('lib').rsplit('-',1)[0] not in normalized:continue
                shutil.copy2(item,dest/item.name);files+=1;total+=item.stat().st_size
            elif item.is_dir() and item.name.rsplit('-',1)[0] in names:
                shutil.copytree(item,dest/item.name,symlinks=False)
                for file in (dest/item.name).rglob('*'):
                    if file.is_file():files+=1;total+=file.stat().st_size
    result.update(complete=True,copied_files=files,copied_bytes=total)
except BaseException as error:
    result.update(exception_type=type(error).__name__,error=str(error));raise
finally:save();print(json.dumps(result))
