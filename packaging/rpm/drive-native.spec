# drive-native RPM packaging (development build).
#
# This spec compiles nothing. It packages the installation tree produced by the
# shared Debian/Ubuntu staging path:
#
#     scripts/build-linux.sh  <build>              # cargo release bins, GIO module, PyInstaller GUI
#     scripts/stage-linux.sh  <build> <stage>      # lay out the packages' files
#     scripts/build-rpm.sh    <output>             # relocate GIO module, then run rpmbuild
#
# The staged tree is passed with --define '_stagedir /absolute/path'. Insisting
# on a pre-staged tree keeps .deb and .rpm packages built from one staging code
# path instead of a second, divergent RPM build recipe. %install only copies
# that tree, relocates the Ubuntu multiarch GIO module to the Fedora/RHEL
# libdir, and strips the three native binaries plus the GIO module.
#
# The .deb and .rpm are independent build instances from the same source commit
# and the same staging scripts: their payload file sets match (the .deb adds
# changelog.Debian.gz/copyright), but native binaries and the PyInstaller
# bundle are not guaranteed byte-identical (build-id/archive metadata differ).

%global debug_package %{nil}

# Pin the payload compression instead of inheriting the build host's rpm macro:
# Ubuntu's rpm defaults to gzip/9 here, which made the same tree 73,526,191 B,
# while w19.xzdio produces 49,213,525 B (-33%) with normal rpm2cpio/rpm -K
# handling. Pinning it also keeps the artifact independent of the builder.
%global _binary_payload w19.xzdio

# The PyInstaller GUI bundle under /usr/lib/drive-native/gui ships a private
# Python/Qt runtime. The deb excludes that tree from dh_strip (-X...); mirror
# that here by disabling the automatic post-install scriptlets (strip, brp-*)
# and stripping only the files we own in %install.
%global __os_install_post %{nil}

# Fedora/RHEL x86_64 GIO module directory. stage-linux.sh installs the probe at
# the Ubuntu multiarch path (/usr/lib/x86_64-linux-gnu/gio/modules) because it
# runs on the Ubuntu build host; %install relocates it to this target path.
%global gio_moduledir /usr/lib64/gio/modules

# Pre-staged build mode: the staging tree is not a tarball source. Let a
# command-line --define '_stagedir ...' win over this default.
%{!?_stagedir:%global _stagedir %{_topdir}/stage}

# The bundled PyInstaller Qt/Python libraries are private copies. Their ELF
# NEEDED entries must not become Requires and their SONAMEs must not become
# Provides: a libQt6Core.so.6() dependency would either be unsatisfiable or drag
# a second, different Qt into the transaction. Requires for the system libraries
# the bundle needs are declared explicitly below.
%global __requires_exclude_from ^/usr/lib/drive-native/gui/.*$
%global __provides_exclude_from ^/usr/lib/drive-native/gui/.*$

Name:           drive-native
Version:        0.1.0~dev20261118.1
Release:        1
Summary:        Native Synology Drive client for Linux
License:        MIT
URL:            https://github.com/Austrust/drive-native
BuildArch:      x86_64

# No BuildRequires: rpmbuild does not compile or bundle anything here. The
# staging tree is produced by the wrapper script on the build host, which owns
# the toolchain requirements (cargo, gcc, cmake, pkg-config, python3, PySide6).
# Requires on the target distro are listed explicitly instead of relying on
# ${shlibs:Depends} (deb) or on the GUI bundle's private libraries.

# Base runtime: glibc provides libc/libm/libpthread/ld-linux; libgcc provides
# libgcc_s. Measured ELF NEEDED for the native binaries is exactly
# libgcc_s.so.1, libm.so.6, libc.so.6 and the dynamic loader, so glibc 2.39
# (the build host baseline) is the effective floor.
Requires:       glibc
Requires:       libgcc
# FUSE mount support and the fusermount3 helper/udev rule set.
Requires:       fuse3
# GIO module host. Fedora/RHEL glib2 ships the cache helper as
# gio-querymodules-64 (renamed from gio-querymodules by its %__isa_bits macro)
# and refreshes /usr/lib64/gio/modules through its own file transfiletrigger;
# the package scriptlets below only add a guarded, idempotent fallback.
Requires:       glib2
# Per-user systemd units are shipped; installing the package needs systemd.
Requires:       systemd
Requires:       ca-certificates
Requires:       hicolor-icon-theme
# Nemo/Nautilus status extensions are Python: they need the GObject bindings.
Requires:       python3-gobject
# GIO volume/mount plumbing used by the probe.
Requires:       gvfs
Requires:       gvfs-fuse

# GUI runtime system libraries. The GUI tree is excluded from automatic
# dependency generation, so these are declared by hand.
#
# Measured on the staged tree (Ubuntu 24.04 PyInstaller onedir): the bundle
# carries its own copies of most of its library stack (Qt6/PySide6, libX11,
# libxkbcommon, fontconfig, dbus-1, xcb-util-*), so these are the only SONAMEs
# the bundle does NOT satisfy itself: libEGL.so.1, libGL.so.1, libdrm.so.2,
# libwayland-client/cursor/egl, libxcb.so.1 (plus glibc, already required).
Requires:       mesa-libGL
Requires:       mesa-libEGL
Requires:       libdrm
Requires:       libwayland-client
Requires:       libwayland-cursor
Requires:       libwayland-egl
Requires:       libxcb

# Declared desktop-runtime contract mirroring the explicit list in
# debian/control. These libraries are currently bundled inside the PyInstaller
# tree, but the demos/gui code may load system copies through GIO/GVFS and the
# packaging must not silently lose a library if PyInstaller stops bundling it.
Requires:       libxkbcommon-x11
Requires:       xcb-util-cursor
Requires:       xcb-util-wm
Requires:       xcb-util-image
Requires:       xcb-util-keysyms
Requires:       xcb-util-renderutil
Requires:       libX11
Requires:       libXext
Requires:       libXrender
Requires:       libXi
Requires:       libSM
Requires:       libICE
Requires:       fontconfig
Requires:       dbus-libs

# Optional desktop integration. Weak dependencies are ignored on systems where
# the package is unavailable (e.g. nemo-python outside Fedora/EPEL).
Recommends:     gnome-keyring
Recommends:     google-noto-sans-cjk-fonts
Recommends:     nemo-python
Recommends:     nautilus-python

%description
Drive Native connects to Synology Drive over the native protocol (no WebDAV,
no vendor daemon) and exposes the remote tree as a FUSE mount with on-demand
hydration, a durable local write queue and a PySide6/Qt desktop client.

The package installs the fsd/worker/setup binaries, a PyInstaller GUI bundle,
the GIO volume probe, Nemo/Nautilus status extensions, hicolor icons, the
desktop entry and per-user systemd units. The user units are installed but not
enabled by package installation; the GUI enables them after account setup, and
removing the package does not delete user configuration or cached content.

This is a development build with open recovery and desktop acceptance gates;
it is not a validated release candidate. The RPM packaging is new in this
release and has not been through a distro release process: it is unsigned and
reuses the Ubuntu 24.04-built staging tree, so its runtime baseline is
glibc 2.39 (Fedora 40+/RHEL 10+). RHEL 8/9 are not supported.

%prep
# Nothing to unpack: the payload is staged outside rpmbuild on purpose.
:

%build
# Nothing to build: cargo/PyInstaller run in scripts/build-linux.sh.
:

%install
rm -rf %{buildroot}
install -d %{buildroot}

stage="%{_stagedir}"
if [ ! -d "$stage/usr" ]; then
    echo "ERROR: pre-staged installation tree not found at $stage" >&2
    echo "ERROR: run scripts/build-rpm.sh (which builds and stages first) or pass" >&2
    echo "ERROR:   --define '_stagedir /absolute/path/to/stage'" >&2
    exit 1
fi
cp -a "$stage"/. %{buildroot}/

# Relocate the GIO volume probe from the Ubuntu multiarch path to the
# Fedora/RHEL x86_64 module path. scripts/build-rpm.sh already does this after
# staging, but a raw stage-linux.sh tree is accepted too.
legacy="%{buildroot}/usr/lib/x86_64-linux-gnu/gio/modules/libdrive-native-probe.so"
if [ -f "$legacy" ]; then
    install -Dm755 "$legacy" "%{buildroot}%{gio_moduledir}/libdrive-native-probe.so"
    rm -f "$legacy"
    rmdir "%{buildroot}/usr/lib/x86_64-linux-gnu/gio/modules" 2>/dev/null || :
    rmdir "%{buildroot}/usr/lib/x86_64-linux-gnu/gio" 2>/dev/null || :
    rmdir "%{buildroot}/usr/lib/x86_64-linux-gnu" 2>/dev/null || :
fi
if [ ! -f "%{buildroot}%{gio_moduledir}/libdrive-native-probe.so" ]; then
    echo "ERROR: GIO module missing from the staged tree" >&2
    exit 1
fi

# Strip only the native Rust binaries and the GIO module. The PyInstaller GUI
# bundle is deliberately left untouched (matches dh_strip -Xusr/lib/drive-native/gui).
for binary in \
    "%{buildroot}/usr/lib/drive-native/drive-native-fsd" \
    "%{buildroot}/usr/lib/drive-native/drive-native-worker" \
    "%{buildroot}/usr/lib/drive-native/drive-native-setup" \
    "%{buildroot}%{gio_moduledir}/libdrive-native-probe.so"
do
    if [ ! -f "$binary" ]; then
        echo "ERROR: expected staged binary missing: $binary" >&2
        exit 1
    fi
    /usr/bin/strip --strip-unneeded "$binary"
done

# Install the user units only; never enable them here (see debian/rules
# override_dh_installsystemduser). No giomodule.cache is shipped: the guarded
# %post/%postun scriptlets rebuild the shared cache from the installed module.
if [ -e "%{buildroot}%{gio_moduledir}/giomodule.cache" ]; then
    echo "ERROR: staged tree must not ship giomodule.cache" >&2
    exit 1
fi

%check
# Fail loudly instead of shipping a package whose embedded identity disagrees
# with the spec/EVR (e.g. when debian/changelog was not bumped first).
grep -Fxq "%{version}" "%{buildroot}/usr/share/doc/drive-native/BUILD_VERSION" || {
    echo "ERROR: BUILD_VERSION does not match Version: %{version}" >&2
    exit 1
}
grep -Eq '^[0-9a-f]{40}$' "%{buildroot}/usr/share/doc/drive-native/SOURCE_COMMIT" || {
    echo "ERROR: SOURCE_COMMIT is not a 40-hex commit id" >&2
    exit 1
}
test -x "%{buildroot}/usr/bin/drive-native"
test -x "%{buildroot}/usr/lib/drive-native/gui/drive-native-gui"
test -f "%{buildroot}%{gio_moduledir}/libdrive-native-probe.so"
test ! -e "%{buildroot}/usr/lib/x86_64-linux-gnu"
for unit in drive-native.service drive-native-unlock.service drive-native@.service drive-native-unlock@.service; do
    test -f "%{buildroot}/usr/lib/systemd/user/$unit"
done

%post
# No giomodule.cache is owned by this package. On Fedora/RHEL glib2 installs
# the helper as gio-querymodules-64 (the bare name does not exist there) and
# refreshes %{gio_moduledir} through its own file transfiletrigger; this call
# is only a guarded, idempotent supplement and must never fail.
query=gio-querymodules-64
command -v "$query" >/dev/null 2>&1 || query=gio-querymodules
command -v "$query" >/dev/null 2>&1 && "$query" %{gio_moduledir} >/dev/null 2>&1 || :

%postun
# Same as the deb postrm guard: refresh only on final removal, not on upgrade
# (%post of the new package already refreshed the cache). glib2's own
# transfiletrigger remains authoritative on Fedora/RHEL.
if [ "$1" -eq 0 ]; then
    query=gio-querymodules-64
    command -v "$query" >/dev/null 2>&1 || query=gio-querymodules
    command -v "$query" >/dev/null 2>&1 && "$query" %{gio_moduledir} >/dev/null 2>&1 || :
fi

%files
%defattr(-,root,root,-)
/usr/bin/drive-native
%dir /usr/lib/drive-native
/usr/lib/drive-native/drive-native-fsd
/usr/lib/drive-native/drive-native-worker
/usr/lib/drive-native/drive-native-setup
/usr/lib/drive-native/gui
%{gio_moduledir}/libdrive-native-probe.so
/usr/share/nemo-python/extensions/drive_native_nemo.py
/usr/share/nautilus-python/extensions/drive_native_nautilus.py
%dir /usr/share/drive-native
%dir /usr/share/drive-native/filemanager
/usr/share/drive-native/filemanager/drive_native_provider.py
/usr/share/icons/hicolor/scalable/emblems/drive-native-*.svg
/usr/share/icons/hicolor/scalable/apps/drive-native.svg
/usr/share/applications/drive-native.desktop
/usr/lib/systemd/user/drive-native.service
/usr/lib/systemd/user/drive-native-unlock.service
/usr/lib/systemd/user/drive-native@.service
/usr/lib/systemd/user/drive-native-unlock@.service
/usr/share/doc/drive-native

%changelog
* Wed Nov 18 2026 Drive Native contributors <noreply@example.invalid> - 0.1.0~dev20261118.1-1
- Initial RPM package, assembled from the same source commit and the same
  staging scripts as the .deb (scripts/build-linux.sh + scripts/stage-linux.sh)
  instead of a second build recipe. The two packages are independent build
  instances: the payload file sets match, but native binaries and the
  PyInstaller bundle are not guaranteed byte-identical.
- Relocate the GIO volume probe from the Ubuntu multiarch module directory to
  /usr/lib64/gio/modules and refresh the GIO module cache from guarded
  %post/%postun scriptlets.
- Install the four per-user systemd units without enabling them; package
  installation never activates a user's mounts.
- Exclude the bundled PyInstaller Qt/Python runtime from automatic dependency
  and provide generation; declare the GUI system libraries explicitly and
  leave the bundle unstripped while stripping the native binaries and module.
- Refresh the GIO module cache through Fedora/RHEL's gio-querymodules-64 with a
  gio-querymodules fallback (glib2 owns the authoritative file transfiletrigger);
  the guarded scriptlets never fail.
- Document that the .deb and .rpm are independent build instances from the same
  source commit and staging scripts: the payload file sets match, but native
  binaries and the PyInstaller bundle are not guaranteed byte-identical.
