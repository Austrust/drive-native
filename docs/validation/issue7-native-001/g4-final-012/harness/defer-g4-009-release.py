import datetime,json,pathlib
run=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-009')
output=run/'coordinator-control.json';assert not output.exists()
value={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'reason':'root requested defer expensive release while investigating unchanged 900ms retry assertions; continue unrelated debug/GUI groups',
 'defer_commands':['kernel-fs-release-*','kernel-runtime-release']}
output.write_text(json.dumps(value,indent=2)+'\n');print(json.dumps(value))
