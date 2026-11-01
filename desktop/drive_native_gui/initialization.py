"""Render observed work, never an estimated completion percentage."""
PHASES = {
    "starting": "正在启动同步进程", "authenticating": "正在认证",
    "configuring": "正在配置传输", "binding": "正在绑定同步目录",
    "catching_up": "正在追赶远端变更", "recovering": "正在准备索引恢复",
    "watermark": "正在取得当前事件水位并核对根目录",
    "seeding": "正在读取目录历史", "enumerating": "正在枚举目录",
    "fencing": "正在核对枚举期间的变更", "importing": "正在提交本地索引",
    "reconciling": "正在协调扫描期间变化的目录",
    "retrying": "本次候选需要重新核对",
    "waiting": "已连接 · 等待远端变更",
}

RETRY_REASONS = {
    "enumeration_collision": "枚举期间名称或身份发生冲突",
    "root_event": "同步根发生变化",
    "directory_event": "目录发生变化，需重新核对范围",
    "filtered_type_event": "发生尚不能局部协调的文件类型变化",
    "type_identity_mismatch": "文件类型与已有身份不一致",
    "parent_unavailable": "变化涉及尚未确认或不可访问的父目录",
    "parent_path_mismatch": "父目录身份与事件路径未能对应",
    "local_listing_denied": "重查目录时访问被拒绝",
    "local_listing_filtered_type": "重查目录时遇到过滤类型",
    "local_listing_changed": "重查列表的目录结构或身份发生变化",
}


def progress_text(progress):
    if not isinstance(progress, dict) or not progress.get("active"):
        return "", ""
    phase = progress.get("phase")
    title = PHASES.get(phase, "正在初始化")
    seconds = max(0, int(progress.get("phase_elapsed_ms", 0))) // 1000
    quiet = max(0, int(progress.get("since_progress_ms") or 0)) // 1000
    if phase == "waiting":
        detail = (f"已应用 {progress.get('events_applied', 0)} 条事件；"
                  "事件通道正在等待通知，这不表示同步停滞。")
        if progress.get("consecutive_changed_without_advance", 0):
            detail += f" 上次连续 {progress['consecutive_changed_without_advance']} 次变更通知后游标未推进。"
        return title, detail
    detail = f"本阶段已用 {seconds} 秒"
    units, items = progress.get("units", 0), progress.get("items", 0)
    if phase == "enumerating":
        detail += f" · 已处理 {units} 个目录，发现 {items} 个条目"
    elif phase == "watermark":
        detail += f" · 已完成 {units} 次边界与身份核对"
    elif phase == "reconciling":
        detail += f" · 已重查 {units} 个受影响目录，候选索引尚未提交"
    elif phase in ("seeding", "fencing"):
        detail += f" · 已读取 {units} 批，{items} 条事件"
    elif phase == "catching_up":
        detail += f" · 游标推进 {progress.get('cursor_advances', 0)} 次，已应用 {progress.get('events_applied', 0)} 条事件"
    if quiet >= 30:
        detail += f" · 距上次完成实际工作 {quiet} 秒"
    if progress.get("consecutive_changed_without_advance", 0):
        detail += f" · 连续 {progress['consecutive_changed_without_advance']} 次变更通知后游标未推进"
    reason = RETRY_REASONS.get(progress.get("last_retry_reason"))
    if reason:
        detail += f" · 已重试 {progress.get('retry_count', 0)} 次：{reason}"
    return title, detail
