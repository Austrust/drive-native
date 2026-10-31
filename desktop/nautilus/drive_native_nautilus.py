"""Nautilus integration: metadata-only asynchronous local D-Bus."""
import sys
from pathlib import Path
import gi
gi.require_version("Nautilus", "4.0")
from gi.repository import Nautilus
# Source checkout or the package's shared, host-neutral provider.
source = Path(__file__).resolve().parents[1] / "filemanager"
shared = source if source.is_dir() else Path("/usr/share/drive-native/filemanager")
sys.path.insert(0, str(shared))
from drive_native_provider import build_provider, NAME, PATH, STATES
DriveNativeExtension, _TaskExtension = build_provider(Nautilus)
