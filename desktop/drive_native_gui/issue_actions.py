"""Bounded, local presentation for recovery. No network or filesystem probes."""
import json
import re
from pathlib import PurePosixPath

NEXT_STEPS = {
    "content_conflict": "保留双方版本；若按钮不可用，先关闭占用此文件的应用，刷新问题后再处理。",
    "remote_deleted": "先查看仍可访问的父文件夹。需要保留本地内容时，使用可用的“保留双方版本”；不要删除草稿或清除缓存。",
    "commit_uncertain": "后台自动只读核对暂停后，仍可随时请求重新核对；重新核对只会再次确认结果，不会重放创建或覆盖服务器。若确认服务器上不存在该条目，可在文件管理器中删除本地文件作废这次未确认的创建，该删除会记录到同步历史。结果确认前不要重复创建、移动或删除同一条目；若仍无进展，复制诊断并提交问题，可暂停同步保留数据。",
    "namespace_conflict": "检查原目录和目标目录是否仍存在、账号是否有权限，再重新核对。若原位置已改变且无法自动确认，复制诊断提交问题；不要用同名新目录冒充原身份。",
    "directory_creation_conflict": "选择可用的“保留双方版本”为本地文件夹换一个名称；不可用时先检查父目录权限并重新核对。",
    "creation_conflict": "使用可用的“保留双方版本”为本地文件换名；不可用时先关闭占用应用，再刷新问题。",
    "creation_identity_mismatch": "先检查原父目录及权限，再重新核对。若同名条目由其他操作创建，保留现状并复制诊断，不要重复上传覆盖。",
    "parent_unavailable": "打开可用的上级目录，检查原父目录是否被移动、删除或撤销权限。恢复原目录或权限后重新核对；不能恢复时复制诊断。",
    "local_content_unavailable": "暂停同步保留现有数据，检查本机磁盘和应用数据目录是否可用，然后复制诊断提交问题。不要清除缓存或重新建任务。",
    "file_in_use": "先保存并关闭正在编辑此文件的应用，再刷新问题或请求重新核对。",
}
TASK_STEPS = {
    "authentication_or_protocol_rejected": "账号认证未通过。使用“解锁 / 重新登录”，并检查账号的 Drive 应用权限。",
    "certificate_or_configuration": "请打开任务的连接证书设置，核对服务器名称与可信证书后再连接；不要关闭证书验证。",
    "trust_settings_not_durable": "证书设置尚未确认保存。请在连接证书设置中重新读取并保存，再重新核对。",
    "worker_image_unavailable": "后台安装组件不完整。请修复安装并正常重启后台；保留原任务、缓存与未上传内容。",
    "native_root_unavailable": "原同步根已不可用或身份发生变化。请先恢复原目录／访问权限，再重新核对；同名新目录不能替代原身份。",
    "native_state_recovery_required": "远端状态需要恢复。可请求重新核对，后台将沿持久检查点恢复；不要清除缓存或重建任务。",
    "native_index_capacity": "目录索引达到处理预算。请复制诊断摘要提交问题；反复登录不能解除预算限制。",
    "native_response_capacity": "远端响应达到处理预算。请复制诊断摘要提交问题；反复登录不能解除预算限制。",
    "network_unavailable": "检查网络和 NAS 可达性，再请求重新连接。本地完整内容与未上传修改保留。",
    "local_or_protocol_error": "同步需要进一步诊断。请复制诊断摘要；可暂停保留数据，并检查本机磁盘是否可用。",
}

RECHECK_MESSAGES = {
    "queued": "已请求后台核对。结果尚未确认；安全核对后，无依赖的正常待同步操作可继续。",
    "already_queued": "核对请求已在队列中，请等待状态更新；这不表示问题已解决。",
    "reconnecting": "已请求重新连接，连接恢复后继续核对；本地修改和挂载保留。",
    "paused": "当前任务已暂停，暂停状态保持。需要继续时，请在任务页选择“恢复同步”。",
    "login_required": "需要重新登录，请使用“解锁 / 重新登录”输入账号密码。",
    "settings_required": "连接设置需要处理。请在任务设置检查证书、服务器及账号权限；此操作不会绕过验证。",
    "stopping": "挂载正在正常停止，暂不安排核对。停止完成后从任务页重新启动。",
    "busy": "后台正提交本地状态，本次未安排核对。请稍后重试；暂停状态和本地数据保持不变。",
}


def recheck_message(data):
    code = data.get("message_code") if isinstance(data, dict) else None
    accepted = code in ("queued", "already_queued", "reconnecting")
    if (not isinstance(code, str) or code not in RECHECK_MESSAGES or type(data.get("schema")) is not int or data.get("schema") != 1
            or data.get("accepted") is not accepted
            or data.get("paused_preserved") is not True):
        return "后台未返回可识别的核对结果，请刷新状态；不能据此认为问题已解决。"
    return RECHECK_MESSAGES[code]


def next_step(row):
    return NEXT_STEPS.get(row.get("reason"),
        "先刷新状态并请求重新核对；如仍未恢复，复制诊断提交问题。可以暂停同步保留本地数据，不要清除缓存。")


def parent_candidates(path):
    # Lexical checks only: never stat/resolve a FUSE path on the GUI thread.
    if not isinstance(path, str) or len(path) > 16384 or "\\" in path or "\0" in path:
        return []
    parts = path.split("/")
    if not path or path.startswith("/") or any(p in ("", ".", "..") for p in parts):
        return []
    parents = PurePosixPath(path).parents[:32]
    result = ["" if str(p) == "." else str(p) for p in parents]
    if "" not in result:
        result.append("")
    return result


SHUTDOWN_PHASES = {
    "idle", "requested", "stopping_workers", "detaching_mount",
    "draining_filesystem", "checkpointing_local", "complete", "failed",
}
# Mirrors drive_native_gui.shutdown.BLOCKING_REASONS; duplicated here so the
# clipboard path stays importable without the Qt shutdown watcher.
SHUTDOWN_BLOCKING_REASONS = {
    "waiting_for_workers", "unmounting_mount", "draining_open_files",
    "persisting_local_changes", "mount_still_present", "unmount_failed",
    "checkpoint_failed", "worker_join_failed",
}
SYNC_STATES = {"idle", "checking", "transferring", "pending", "paused",
               "needs_attention", "unknown"}
CACHE_STATES = {"unknown", "online_only", "partial", "complete", "pinned"}
DIAGNOSTIC_TOKEN = re.compile(r"[a-z0-9_]{1,64}\Z")


def diagnostic_text(overview, row=None):
    """Explicit whitelist: no account, path, IDs, tokens, or raw backend errors."""
    overview = overview if isinstance(overview, dict) else {}
    row = row if isinstance(row, dict) else {}
    data = {"format": "drive-native-issue-diagnostic-1"}
    allowed = {
        "connection": {"locked", "connecting", "online", "offline", "paused", "needs_attention", "stopping"},
        "error": {"worker_image_unavailable", "authentication_or_protocol_rejected", "trust_settings_not_durable", "certificate_or_configuration", "native_root_unavailable", "native_index_capacity", "native_response_capacity", "native_state_recovery_required", "network_unavailable", "local_or_protocol_error"},
    }
    for key, choices in allowed.items():
        value = overview.get(key)
        if isinstance(value, str):
            data[key] = value if value in choices else "unrecognized"
    for key in ("mounted", "preview_only"):
        if type(overview.get(key)) is bool:
            data[key] = overview[key]
    for key in ("pending_operations", "download_errors"):
        value = overview.get(key)
        if type(value) is int and 0 <= value < 1 << 64:
            data[key] = value
    health = overview.get("queue_health")
    if isinstance(health, dict):
        data["queue_health"] = {k: health[k] for k in ("namespace_uncertain", "upload_uncertain", "namespace_unfinished", "upload_unfinished")
                                if type(health.get(k)) is int and 0 <= health[k] < 1 << 64}
    data["upload_block_present"] = bool(overview.get("upload_block"))
    # Cooperative stop facts: phase and a fixed blocker enum only.  A missing
    # blocking_reason is expected on older backends and is simply omitted.
    shutdown = overview.get("shutdown")
    if isinstance(shutdown, dict):
        closed = {}
        phase = shutdown.get("phase")
        if phase in SHUTDOWN_PHASES:
            closed["phase"] = phase
        reason = shutdown.get("blocking_reason")
        if reason in SHUTDOWN_BLOCKING_REASONS:
            closed["blocking_reason"] = reason
        elif reason is not None:
            closed["blocking_reason"] = "unrecognized"
        for key in ("elapsed_ms", "stage_elapsed_ms"):
            value = shutdown.get(key)
            if type(value) is int and 0 <= value < 1 << 62:
                closed[key] = value
        if type(shutdown.get("overdue")) is bool:
            closed["overdue"] = shutdown["overdue"]
        if closed:
            data["shutdown"] = closed
    diagnostics = overview.get("connection_diagnostics")
    if isinstance(diagnostics, dict):
        closed = {}
        for key in ("load_failed", "persistence_failed"):
            if type(diagnostics.get(key)) is bool:
                closed[key] = diagnostics[key]
        for key in ("last_transition", "last_failure"):
            item = diagnostics.get(key)
            if isinstance(item, dict):
                safe = {}
                for name in ("phase", "category", "recovery"):
                    token = item.get(name)
                    if isinstance(token, str) and DIAGNOSTIC_TOKEN.fullmatch(token):
                        safe[name] = token
                if safe:
                    closed[key] = safe
        if closed:
            data["connection_diagnostics"] = closed
    kind = overview.get("background_error_kind")
    if isinstance(kind, str) and DIAGNOSTIC_TOKEN.fullmatch(kind):
        data["background_error_kind"] = kind
    since = overview.get("background_error_since_ms")
    if type(since) is int and 0 <= since < 1 << 62:
        data["background_error_since_ms"] = since
    sync_state = overview.get("sync_state")
    if sync_state in SYNC_STATES:
        data["sync_state"] = sync_state
    count = overview.get("issue_count")
    if type(count) is int and 0 <= count < 1 << 64:
        data["issue_count"] = count
    cache = overview.get("cache_availability")
    if isinstance(cache, dict):
        closed = {}
        state = cache.get("state")
        if state in CACHE_STATES:
            closed["state"] = state
        elif state is not None:
            closed["state"] = "unrecognized"
        for key in ("offline", "pinned", "total"):
            value = cache.get(key)
            if type(value) is int and 0 <= value < 1 << 64:
                closed[key] = value
        if type(cache.get("checked")) is bool:
            closed["checked"] = cache["checked"]
        if closed:
            data["cache_availability"] = closed
    if row:
        data["issue_reason"] = row.get("reason") if row.get("reason") in NEXT_STEPS else "unrecognized"
        for key in ("directory", "can_keep_both", "can_keep_directory"):
            if type(row.get(key)) is bool:
                data[key] = row[key]
        # Additive reconcile budget fields; absent on older backends.
        if type(row.get("reconcile_exhausted")) is bool:
            data["reconcile_exhausted"] = row["reconcile_exhausted"]
        attempts = row.get("reconcile_attempts")
        if type(attempts) is int and 0 <= attempts < 1 << 32:
            data["reconcile_attempts"] = attempts
    return json.dumps(data, ensure_ascii=False, indent=2)
