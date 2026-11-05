"""Load the coordinator's frozen host-space reader without invoking its main."""
import hashlib
import importlib.util
from pathlib import Path
import sys

def load_host_space(path):
    path = Path(path).resolve()
    assert path.is_file(), 'frozen host-space helper required'
    with path.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location('native_chain_host_space', path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    assert callable(module.host_c_space)
    def observe(minimum_gib):
        try:
            value = module.host_c_space()
            assert isinstance(value, dict)
        except Exception as error:
            value = {'free_bytes': None, 'error_type': type(error).__name__, 'error': str(error)}
        minimum = minimum_gib * 1024 ** 3
        free = value.get('free_bytes')
        return {'helper_path': str(path), 'helper_sha256': digest,
                'minimum_free_bytes': minimum, 'minimum_free_gib': minimum_gib,
                'observation': value, 'passed': isinstance(free, int) and free >= minimum}
    return observe
