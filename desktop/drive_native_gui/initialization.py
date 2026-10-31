"""Render observed work, never an estimated completion percentage."""
PHASES = {
    "starting": "正在启动同步进程", "authenticating": "正在认证",
    "configuring": "正在配置传输", "binding": "正在绑定同步目录",
    "catching_up": "正在追赶远端变更", "recovering": "正在准备索引恢复",
    "seeding": "正在读取目录历史", "enumerating": "正在枚举目录",
    "fencing": "正在核对枚举期间的变更", "importing": "正在提交本地索引",
    "waiting": "已连接 · 等待远端变更",
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
    elif phase in ("seeding", "fencing"):
        detail += f" · 已读取 {units} 批，{items} 条事件"
    elif phase == "catching_up":
        detail += f" · 游标推进 {progress.get('cursor_advances', 0)} 次，已应用 {progress.get('events_applied', 0)} 条事件"
    if quiet >= 30:
        detail += f" · 距上次完成实际工作 {quiet} 秒"
    if progress.get("consecutive_changed_without_advance", 0):
        detail += f" · 连续 {progress['consecutive_changed_without_advance']} 次变更通知后游标未推进"
    return title, detail
