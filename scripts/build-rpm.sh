#!/usr/bin/env bash
# Build the experimental drive-native RPM.
#
# The RPM is assembled from the same staging tree as the .deb: this script runs
# scripts/build-linux.sh (cargo release binaries, GIO module, PyInstaller GUI)
# and scripts/stage-linux.sh, then relocates the GIO module to the Fedora/RHEL
# x86_64 module directory and invokes rpmbuild -bb on packaging/rpm/drive-native.spec.
# rpmbuild itself never compiles anything (see the spec header).
#
# usage:
#   GUI_PYTHON=/abs/.venv-build/bin/python \
#   DRIVE_NATIVE_SOURCE_COMMIT=$(git rev-parse HEAD) \
#   bash scripts/build-rpm.sh /absolute/output/dir
#
# The output directory receives:
#   build/            build-linux.sh output (bins + GUI onedir + gio module)
#   stage/            installation tree handed to rpmbuild via _stagedir
#   rpmbuild/         rpmbuild topdir
#   rpm-build.log     complete rpmbuild log
#   drive-native-<EVR>.<arch>.rpm (+ printed NEVRA and sha256)
set -euo pipefail

output=${1:?usage: scripts/build-rpm.sh /absolute/output/dir}
case "$output" in
    /*) ;;
    *) printf '%s\n' 'Output directory must be absolute' >&2; exit 2 ;;
esac
if [ "$output" = / ]; then
    printf '%s\n' 'Refusing to use / as the output directory' >&2
    exit 2
fi
: "${GUI_PYTHON:?Set GUI_PYTHON to the locked build venv python}"
: "${DRIVE_NATIVE_SOURCE_COMMIT:?Set DRIVE_NATIVE_SOURCE_COMMIT to the source commit}"
case "$DRIVE_NATIVE_SOURCE_COMMIT" in
    *[!0-9a-f]*|'')
        printf '%s\n' "DRIVE_NATIVE_SOURCE_COMMIT must be lowercase 40-hex, got: $DRIVE_NATIVE_SOURCE_COMMIT" >&2
        exit 2
        ;;
esac
if [ "${#DRIVE_NATIVE_SOURCE_COMMIT}" -ne 40 ]; then
    printf '%s\n' "DRIVE_NATIVE_SOURCE_COMMIT must be 40 hex characters, got ${#DRIVE_NATIVE_SOURCE_COMMIT}" >&2
    exit 2
fi

project=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project"

spec=packaging/rpm/drive-native.spec
[ -f "$spec" ] || { printf '%s\n' "Missing spec: $project/$spec" >&2; exit 1; }
version=$(awk '/^Version:/ {print $2; exit}' "$spec")
if [ -z "$version" ]; then
    printf '%s\n' "Cannot read Version from $spec" >&2
    exit 1
fi
# stage-docs.py takes BUILD_VERSION from debian/changelog, so the changelog must
# be bumped before the staging tree is produced; the spec %check enforces this
# too, and this earlier check gives a clearer error.
if command -v dpkg-parsechangelog >/dev/null 2>&1; then
    changelog_version=$(dpkg-parsechangelog -S Version)
    if [ "$changelog_version" != "$version" ]; then
        printf '%s\n' \
            "Version mismatch: spec Version=$version but debian/changelog is $changelog_version" \
            'Bump debian/changelog before building the RPM (source identity and BUILD_VERSION).' >&2
        exit 1
    fi
fi

for tool in rpmbuild rpm rpm2cpio cpio sha256sum strip objdump; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s\n' "Missing required tool: $tool" >&2
        exit 1
    fi
done
[ -x "$GUI_PYTHON" ] || { printf '%s\n' "GUI_PYTHON is not executable: $GUI_PYTHON" >&2; exit 1; }

mkdir -p "$output"
work=$(CDPATH='' cd -- "$output" && pwd)
build=$work/build
stage=$work/stage
topdir=$work/rpmbuild
log=$work/rpm-build.log
module=libdrive-native-probe.so
legacy=$stage/usr/lib/x86_64-linux-gnu/gio/modules/$module
target=$stage/usr/lib64/gio/modules/$module

echo '== drive-native RPM build =='
echo "PROJECT=$project"
echo "OUTPUT=$work"
echo "GUI_PYTHON=$GUI_PYTHON"
echo "SOURCE_COMMIT=$DRIVE_NATIVE_SOURCE_COMMIT"
echo "SPEC_VERSION=$version"
echo "RPMBUILD=$(${RPMBUILD:-rpmbuild} --version)"
echo "HOST=$(hostname)"
echo "GLIBC=$(ldd --version | awk 'NR==1 {print $NF}')"

# Only remove the directories this script owns; never the whole output dir.
rm -rf -- "$build" "$stage" "$topdir"
rm -f -- "$work"/*.rpm

echo '== [1/4] build-linux.sh (cargo release bins + GIO module + PyInstaller GUI) =='
bash scripts/build-linux.sh "$build"

echo '== [2/4] stage-linux.sh (shared deb/rpm installation tree) =='
bash scripts/stage-linux.sh "$build" "$stage"

echo '== [3/4] relocate GIO module for Fedora/RHEL =='
if [ ! -f "$legacy" ]; then
    printf '%s\n' "stage-linux.sh did not install $module at $legacy" >&2
    exit 1
fi
install -Dm755 "$legacy" "$target"
rm -f "$legacy"
for dir in \
    "$stage/usr/lib/x86_64-linux-gnu/gio/modules" \
    "$stage/usr/lib/x86_64-linux-gnu/gio" \
    "$stage/usr/lib/x86_64-linux-gnu"
do
    rmdir --ignore-fail-on-non-empty -- "$dir" 2>/dev/null || :
done
[ -f "$target" ] || { printf '%s\n' 'GIO module relocation failed' >&2; exit 1; }
[ ! -e "$stage/usr/lib/x86_64-linux-gnu" ] || { printf '%s\n' 'Ubuntu multiarch path still staged' >&2; exit 1; }
echo "GIO_MODULE=$target"
echo "STAGED_FILES=$(find "$stage" -type f | wc -l)"

echo '== [4/4] rpmbuild -bb (pre-staged mode, no compilation) =='
mkdir -p "$topdir"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}
install -m644 "$spec" "$topdir/SPECS/drive-native.spec"
set +e
rpmbuild -bb "$topdir/SPECS/drive-native.spec" \
    --define "_topdir $topdir" \
    --define "_stagedir $stage" 2>&1 | tee "$log"
rc=${PIPESTATUS[0]}
set -e
if [ "$rc" -ne 0 ]; then
    echo "RPMBUILD FAILED (rc=$rc); log: $log" >&2
    tail -40 "$log" >&2 || :
    exit "$rc"
fi

rpm_path=$(find "$topdir/RPMS" -type f -name '*.rpm' | sort | head -n 1)
[ -n "$rpm_path" ] || { printf '%s\n' 'rpmbuild produced no .rpm' >&2; exit 1; }
rpm_name=$(basename "$rpm_path")
rpm_file=$work/$rpm_name
cp -f "$rpm_path" "$rpm_file"

nevra=$(rpm -qp --qf '%{NAME}-%{VERSION}-%{RELEASE}.%{ARCH}' "$rpm_file")
[ "$nevra" = "drive-native-$version-1.x86_64" ] || {
    printf '%s\n' "Unexpected NEVRA: $nevra" >&2
    exit 1
}

# Guard the packaging decisions that are easy to regress silently. The query
# output is captured before filtering: piping `rpm -qp...` directly into
# `grep -q` makes grep exit on the first match and can kill rpm with SIGPIPE,
# which pipefail would then report as a failure.
rpm_files=$(rpm -qpl "$rpm_file")
rpm_requires=$(rpm -qp --requires "$rpm_file")
rpm_provides=$(rpm -qp --provides "$rpm_file")

if grep -Eiq 'libqt|pyside|shiboken|libpython|qt6|python3dist' <<<"$rpm_requires"; then
    echo 'ERROR: bundled GUI runtime leaked into Requires:' >&2
    grep -Ei 'libqt|pyside|shiboken|libpython|qt6|python3dist' <<<"$rpm_requires" >&2
    exit 1
fi
if grep -Eiq 'libqt|pyside|shiboken|libpython|qt6|python3dist' <<<"$rpm_provides"; then
    echo 'ERROR: bundled GUI runtime leaked into Provides:' >&2
    grep -Ei 'libqt|pyside|shiboken|libpython|qt6|python3dist' <<<"$rpm_provides" >&2
    exit 1
fi
# Only the packaged *path* is checked, not file names: the PyInstaller bundle
# legitimately contains names such as _cffi_backend.cpython-312-x86_64-linux-gnu.so.
if grep -q '^/usr/lib/x86_64-linux-gnu/' <<<"$rpm_files"; then
    echo 'ERROR: Ubuntu multiarch path leaked into the package' >&2
    grep '^/usr/lib/x86_64-linux-gnu/' <<<"$rpm_files" >&2
    exit 1
fi
for dep in glibc libgcc fuse3 glib2 systemd ca-certificates hicolor-icon-theme \
           python3-gobject gvfs gvfs-fuse mesa-libGL mesa-libEGL libdrm \
           libwayland-client libwayland-cursor libwayland-egl libxcb \
           libxkbcommon-x11 xcb-util-cursor libX11 fontconfig dbus-libs; do
    if ! grep -qx "$dep" <<<"$rpm_requires"; then
        printf '%s\n' "ERROR: declared Requires missing from package: $dep" >&2
        exit 1
    fi
done
for pattern in \
    '^/usr/bin/drive-native$' \
    '^/usr/lib/drive-native/drive-native-fsd$' \
    '^/usr/lib/drive-native/gui/drive-native-gui$' \
    '^/usr/lib64/gio/modules/libdrive-native-probe\.so$' \
    '^/usr/lib/systemd/user/drive-native\.service$' \
    '^/usr/share/doc/drive-native/SOURCE_COMMIT$'
do
    if ! grep -Eq "$pattern" <<<"$rpm_files"; then
        printf '%s\n' "ERROR: expected packaged file missing: $pattern" >&2
        exit 1
    fi
done

# Verify the embedded identity without installing the package.
extract=$(mktemp -d "${TMPDIR:-/tmp}/drive-native-rpm.XXXXXXXX")
trap 'rm -rf -- "$extract"' EXIT
rpm2cpio "$rpm_file" | (cd "$extract" && cpio -idm --quiet)
embedded_commit=$(cat "$extract/usr/share/doc/drive-native/SOURCE_COMMIT")
embedded_version=$(cat "$extract/usr/share/doc/drive-native/BUILD_VERSION")
[ "$embedded_commit" = "$DRIVE_NATIVE_SOURCE_COMMIT" ] || {
    printf '%s\n' "ERROR: embedded SOURCE_COMMIT=$embedded_commit" >&2
    exit 1
}
[ "$embedded_version" = "$version" ] || {
    printf '%s\n' "ERROR: embedded BUILD_VERSION=$embedded_version (spec $version)" >&2
    exit 1
}

echo
echo '== NEEDED (native binaries and GIO module; evidence for RPM_INTEGRATION.md) =='
for binary in \
    usr/lib/drive-native/drive-native-fsd \
    usr/lib/drive-native/drive-native-worker \
    usr/lib/drive-native/drive-native-setup \
    usr/lib64/gio/modules/libdrive-native-probe.so
do
    printf '%s:' "$binary"
    objdump -p "$extract/$binary" | awk '/NEEDED/ {printf " %s", $2}'
    echo
done
echo "GUI_ELF_FILES=$(find "$extract/usr/lib/drive-native/gui" -type f -print0 \
    | xargs -0 file | grep -c 'ELF' || true)"
echo "PACKAGED_FILES=$(wc -l <<<"$rpm_files")"
echo "BUILD_ID_LINKS=$(grep -c '^/usr/lib/.build-id/' <<<"$rpm_files" || true)"

echo
echo '== RPM build complete =='
echo "RPM_NEVRA=$nevra"
echo "RPM_FILE=$rpm_file"
echo "RPM_SIZE_BYTES=$(stat -c %s "$rpm_file")"
sha=$(sha256sum "$rpm_file" | awk '{print $1}')
echo "RPM_SHA256=$sha"
echo "SOURCE_COMMIT=$DRIVE_NATIVE_SOURCE_COMMIT"
echo "BUILD_VERSION=$version"
# GitHub release uploads normalize '~' in asset names; only a hint for the Lead.
echo "GITHUB_ASSET_NAME_HINT=$(printf '%s' "$rpm_name" | tr '~' '.')"
echo
echo 'Explicit and generated dependencies:'
printf '%s\n' "$rpm_requires"
