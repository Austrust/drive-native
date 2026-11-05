"""Record coordinator's supersession boundary; do not alter completed results."""
import json,pathlib
run=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-011')
path=run/'coordinator-control.json';assert not path.exists()
value={'reason':'Coordinator confirmed new typed Read-denial scheduling defect; preserve current workspace-default completion, defer remaining groups, reproduce with real TLS worker chain before a new frozen candidate.',
    'defer_commands':['vendored-fuser-wire','kernel-fs-*','kernel-runtime-*','runtime-private-dbus','gui-*','provider-*','unified-ui-render']}
path.write_text(json.dumps(value,indent=2)+'\n');print(json.dumps(value))
