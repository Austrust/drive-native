"""Independent functional / safety verification on synthetic FUSE mounts.

Verifier-owned script (not imported from the product repo harness except for the
verifier's own scan/audit implementation in verify-fsearch.py).

Covers, on real kernel mounts and with the ORIGINAL upstream FSearch 0.3.1:
  * two mounts in parallel: a long scan on mount A must not block mount B
  * cancellation of an actively scanning original FSearch run, then mount stays
    usable and a subsequent full scan still matches truth (no leaked view/lock)
  * local edits through the writable mount: write->immediate read, dirty draft
    size/mtime, rename, unlink, then a full FSearch scan matches updated truth
  * zero body hydration attempts throughout
  * repeated opendir/releasedir cycles on a wide directory (view release smoke)
  * normal stops: fixture exit 0 and no mount left behind
"""
import argparse
import json
import os
import pathlib
import signal
import subprocess
import sys
import time

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import importlib.util

_spec = importlib.util.spec_from_file_location(
    "verify_fsearch", pathlib.Path(__file__).resolve().parent / "run-paired.py"
)
vf = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(vf)
vf.load_context = vf.load_sample
vf.competitors = vf.processes


def save(result, path):
    path.write_text(json.dumps(result, indent=2) + "\n")


def profile_env(profile, mount):
    for name in ["config/fsearch", "data/fsearch", "cache", "runtime"]:
        (profile / name).mkdir(parents=True, exist_ok=True, mode=0o700)
    (profile / "config/fsearch/fsearch.conf").write_text(
        "[Database]\n"
        f"folder_1_path={mount}\n"
        "folder_1_active=true\n"
        "folder_1_one_file_system=false\n"
        "folder_1_monitor=false\n"
        "folder_1_scan_after_launch=false\n"
        "folder_1_rescan_after=0\n"
    )
    env = os.environ.copy()
    for key in ["DBUS_SESSION_BUS_ADDRESS", "DISPLAY", "WAYLAND_DISPLAY", "G_MESSAGES_DEBUG"]:
        env.pop(key, None)
    env.update(
        XDG_CONFIG_HOME=str(profile / "config"),
        XDG_DATA_HOME=str(profile / "data"),
        XDG_CACHE_HOME=str(profile / "cache"),
        XDG_RUNTIME_DIR=str(profile / "runtime"),
        GSETTINGS_BACKEND="memory",
        NO_AT_BRIDGE="1",
    )
    return env


def start_fixture(binary, run, name, mode, groups, files, wide, diagnostics_required=True):
    case = run / name
    case.mkdir()
    stderr = (case / "fixture.stderr").open("w")
    env = os.environ.copy()
    env["DRIVE_NATIVE_FS_DIAGNOSTICS"] = "1"
    process = subprocess.Popen(
        [str(binary), str(case / "fixture"), mode, str(groups), str(files), str(wide)],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
        text=True, bufsize=1, env=env, start_new_session=True,
    )
    mount = (case / "fixture/mount").resolve()
    try:
        ready = vf.response(process, 300)
        if not ready.get("ready"):
            raise AssertionError(f"fixture {name} not ready: {ready}")
        truth = {
            row["path"]: {key: row[key] for key in ["directory", "size", "mtime"]}
            for row in json.loads((case / "fixture/truth.json").read_text())
        }
        sample = vf.command(process, "sample")
        if diagnostics_required and "diagnostics" not in sample:
            raise AssertionError("fixture was not built with drive_fs_diagnostics")
    except BaseException:
        save(vf.stop_fixture(process, mount), case / "startup-cleanup.json")
        stderr.close()
        raise
    return {
        "name": name, "case": case, "mount": mount, "truth": truth,
        "process": process, "stderr": stderr, "mode": mode, "ready": ready,
    }


def stop_fixture(item, result):
    outcome = vf.stop_fixture(item["process"], item["mount"])
    item["stderr"].close()
    record = {"case": item["name"], **outcome}
    result["normal_stops"].append(record)
    if not outcome["normal"] or outcome["forced_cleanup"] or outcome["residual_mount"]:
        raise AssertionError(f"fixture {item['name']} abnormal exit: {record}")
    return record


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--run", required=True, type=pathlib.Path)
    parser.add_argument("--fixture", required=True, type=pathlib.Path)
    parser.add_argument("--fsearch", required=True, type=pathlib.Path)
    parser.add_argument("--audit", required=True, type=pathlib.Path)
    parser.add_argument("--g4-functional-authorized", action="store_true")
    parser.add_argument("--candidate-commit", required=True)
    args = parser.parse_args()
    if not args.g4_functional_authorized:
        raise SystemExit("explicit G4 functional authorization required")
    run = args.run.resolve()
    if not run.is_relative_to(pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')):
        raise SystemExit('functional runs must stay inside the private validation root')
    assert vf.sha(args.fsearch)==vf.EXPECTED['fsearch']
    assert vf.sha(args.audit)==vf.EXPECTED['audit']
    if run.exists():
        raise SystemExit("run directory must be new")
    run.mkdir(parents=True, mode=0o700)
    result = {
        "complete": False,
        "kind": "independent_verifier_functional_and_safety",
        "original_upstream_fsearch": True,
        "no_nas": True,
        "candidate_commit": args.candidate_commit,
        "performance_evidence": False,
        "timing_scope": "functional diagnostics only; concurrent G4 work is permitted; never pooled with G5",
        "harness_sha256": vf.sha(pathlib.Path(__file__)),
        "scan_helper_sha256": vf.sha(pathlib.Path(__file__).resolve().parent / "run-paired.py"),
        "binaries": {
            "fixture": vf.sha(args.fixture), "fsearch": vf.sha(args.fsearch),
            "audit": vf.sha(args.audit),
        },
        "checks": {},
        "normal_stops": [],
        "started": vf.load_context(),
        "competitors_before": vf.competitors(),
    }
    save(result, run / "result.json")
    big = small = None
    try:
        big = start_fixture(args.fixture, run, "main", "fuse-writable", 500, 20, 1000)
        small = start_fixture(args.fixture, run, "other", "fuse", 2, 2, 2)
        result["checks"]["two_mounts_ready"] = {
            "main": {"entries": len(big["truth"]), "mount": str(big["mount"]), "ready":big["ready"]},
            "other": {"entries": len(small["truth"]), "mount": str(small["mount"]), "ready":small["ready"]},
        }
        save(result, run / "result.json")

        # ---- 1. active original scan on mount A must not block mount B -----
        profile = big["case"] / "profile"
        env = profile_env(profile, big["mount"])
        scan_log = (big["case"] / "cancelled.log").open("w")
        scan = subprocess.Popen(
            ["dbus-run-session", "--", "xvfb-run", "-a",
             "--server-args=-screen 0 1280x800x24 -nolisten tcp",
             str(args.fsearch), "--update-database"],
            env=env, stdout=scan_log, stderr=subprocess.STDOUT, start_new_session=True,
        )
        other_latencies = []
        active_opendirs = 0
        started_active = False
        deadline = time.monotonic() + 60
        try:
            while time.monotonic() < deadline:
                if scan.poll() is not None:
                    raise AssertionError("scan finished before active-cancel check")
                sample = vf.command(big["process"], "sample")
                active_opendirs = sum(
                    s["count"] for s in sample.get("diagnostics", []) if s["stage"] == "OpendirRequests"
                )
                t = time.monotonic()
                if os.stat(small["mount"] / "d00000/f00000.dat").st_size != 1234:
                    raise AssertionError("other mount returned wrong size during scan")
                if "d00000" not in os.listdir(small["mount"]):
                    raise AssertionError("other mount listing incomplete during scan")
                other_latencies.append(time.monotonic() - t)
                if active_opendirs >= 3:
                    started_active = True
                    break
                time.sleep(0.02)
            if not started_active:
                raise AssertionError("no actual directory requests observed on mount A")
            result["checks"]["parallel_mount_not_blocked"] = {
                "method": "stat+listdir on mount B while original FSearch actively scans mount A",
                "observed_opendirs_on_A": active_opendirs,
                "other_mount_samples": len(other_latencies),
                "other_mount_max_seconds": max(other_latencies),
                "other_mount_median_seconds": sorted(other_latencies)[len(other_latencies) // 2],
            }
            save(result, run / "result.json")

            # ---- 2. cancel actively running scan -------------------------
            os.killpg(scan.pid, signal.SIGTERM)
            cancel_rc = scan.wait(timeout=30)
            result["checks"]["cancelled_active_original_scan"] = {
                "method": "SIGTERM to verifier-owned private scan process group; not GUI cancel button",
                "observed_opendirs": active_opendirs,
                "exit_code": cancel_rc,
            }
        finally:
            if scan.poll() is None:
                os.killpg(scan.pid, signal.SIGTERM)
                scan.wait(timeout=30)
            scan_log.close()

        # mount A must remain usable after cancellation
        after_cancel_entries = len(os.listdir(big["mount"]))
        stat_after = os.stat(big["mount"] / "d00000/f00000.dat")
        sample_after_cancel = vf.command(big["process"], "sample")
        result["checks"]["mount_after_cancel"] = {
            "root_entries": after_cancel_entries,
            "file_size": stat_after.st_size,
            "hydration_attempts": sample_after_cancel["hydration_attempts"],
        }
        # 500 group dirs + wide + deep00 = 502 entries under the fixture root.
        if after_cancel_entries != 502:
            raise AssertionError(f"root listing after cancel: {after_cancel_entries}")
        if stat_after.st_size != 1234:
            raise AssertionError("stat after cancel returned wrong size")
        if sample_after_cancel["hydration_attempts"] != 0:
            raise AssertionError("hydration attempted during cancel/after-cancel")
        save(result, run / "result.json")

        # ---- 3. restart full scan after cancellation ---------------------
        before = vf.command(big["process"], "sample")
        restart = vf.scan(
            args.fsearch, args.audit, big["mount"], profile,
            big["case"] / "restart.log", big["truth"],
        )
        after = vf.command(big["process"], "sample")
        restart.update(
            hydration_delta=after["hydration_attempts"] - before["hydration_attempts"]
        )
        if restart["hydration_delta"] != 0:
            raise AssertionError("hydration during restart full scan")
        result["checks"]["restart_full_scan"] = restart
        save(result, run / "result.json")

        # ---- 4. local edits through the writable mount -------------------
        old = "d00000/f00000.dat"
        renamed = "d00000/renamed.dat"
        removed = "d00000/f00001.dat"
        new = "d00000/local-draft.dat"
        body = b"local unsent draft for the original FSearch index"
        os.rename(big["mount"] / old, big["mount"] / renamed)
        big["truth"][renamed] = big["truth"].pop(old)
        os.unlink(big["mount"] / removed)
        del big["truth"][removed]
        with (big["mount"] / new).open("wb") as out:
            out.write(body)
            out.flush()
            os.fsync(out.fileno())
        os.utime(big["mount"] / new, (1700000123, 1700000123))
        big["truth"][new] = {"directory": False, "size": len(body), "mtime": 1700000123}
        # immediate read-back and dirty-draft size/mtime through the same mount
        read_back = (big["mount"] / new).read_bytes()
        draft_stat = os.stat(big["mount"] / new)
        renamed_stat = os.stat(big["mount"] / renamed)
        if read_back != body:
            raise AssertionError("write->immediate read returned different bytes")
        if draft_stat.st_size != len(body) or int(draft_stat.st_mtime) != 1700000123:
            raise AssertionError(
                f"dirty draft local size/mtime mismatch: {draft_stat.st_size}/{draft_stat.st_mtime}"
            )
        if not os.path.isdir(big["mount"] / "d00000"):
            raise AssertionError("renamed parent directory missing")
        result["checks"]["local_edits"] = {
            "write_immediate_read_ok": True,
            "draft_body_bytes": len(body),
            "dirty_draft_size": draft_stat.st_size,
            "dirty_draft_mtime": int(draft_stat.st_mtime),
            "rename_visible": renamed_stat.st_size == 1234,
            "delete_visible": not (big["mount"] / removed).exists(),
        }
        save(result, run / "result.json")

        # ---- 5. full scan sees exactly the edited truth ------------------
        before = vf.command(big["process"], "sample")
        edited = vf.scan(
            args.fsearch, args.audit, big["mount"], profile,
            big["case"] / "local-changes.log", big["truth"],
        )
        after = vf.command(big["process"], "sample")
        edited.update(hydration_delta=after["hydration_attempts"] - before["hydration_attempts"])
        if edited["hydration_delta"] != 0:
            raise AssertionError("hydration during edited-tree full scan")
        if (big["mount"] / new).read_bytes() != body:
            raise AssertionError("dirty draft content changed by scan")
        result["checks"]["local_changes_full_scan"] = edited
        save(result, run / "result.json")

        # ---- 6. opendir/releasedir cycles on a wide directory ------------
        rss_before = vf.command(big["process"], "sample").get("diagnostics")
        times = []
        for _ in range(200):
            t = time.monotonic()
            entries = os.listdir(big["mount"] / "wide")
            if len(entries) != 1000:
                raise AssertionError(f"wide listing size {len(entries)}")
            times.append(time.monotonic() - t)
        sample_cycles = vf.command(big["process"], "sample")
        result["checks"]["releasedir_cycles"] = {
            "cycles": 200, "entries_per_listing": 1000,
            "max_seconds": max(times), "median_seconds": sorted(times)[100],
            "hydration_attempts": sample_cycles["hydration_attempts"],
        }
        if sample_cycles["hydration_attempts"] != 0:
            raise AssertionError("hydration during opendir cycles")
        save(result, run / "result.json")

        # ---- 7. hydrations still zero on both mounts ---------------------
        samples = {
            "main": vf.command(big["process"], "sample"),
            "other": vf.command(small["process"], "sample"),
        }
        result["checks"]["hydrations"] = {
            key: value["hydration_attempts"] for key, value in samples.items()
        }
        if any(value["hydration_attempts"] != 0 for value in samples.values()):
            raise AssertionError(f"hydration attempts observed: {samples}")
        result["checks"]["final_diagnostics"] = samples
        assert any(row['stage']=='IndexHits' and row['count']>0 for row in samples['main']['diagnostics']), 'functional run must exercise NSI'
        result["ended"] = vf.load_context()
        result["complete"] = True
        save(result, run / "result.json")
    except BaseException as error:
        result["failure_type"] = type(error).__name__
        result["failure_message"] = str(error)
        result["ended"] = vf.load_context()
        save(result, run / "result.json")
        raise
    finally:
        for item in [big, small]:
            if item is None:
                continue
            try:
                stop_fixture(item, result)
            except BaseException as stop_error:
                result.setdefault("stop_errors", []).append(str(stop_error))
                result['complete'] = False
            save(result, run / "result.json")
    print(json.dumps({"complete":result['complete'],"run":str(run)}))
    return 0 if result['complete'] else 1


if __name__ == "__main__":
    sys.exit(main())
