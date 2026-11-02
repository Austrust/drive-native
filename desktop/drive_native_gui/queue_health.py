"""Presentation of confirmed queue counts, independent of network connectivity."""


def uncertain_operations(overview):
    health = overview.get("queue_health") if isinstance(overview, dict) else None
    if not isinstance(health, dict):
        return 0
    return sum(value for key in ("namespace_uncertain", "upload_uncertain")
               if type(value := health.get(key)) is int and 0 <= value < 1 << 64)


def queue_notice(overview):
    count = uncertain_operations(overview)
    if not count:
        return ""
    return (f"{count} 项操作的服务器执行结果尚未确认。"
            "本地修改已保留，相关操作等待核对；请查看“同步问题”。")


def queue_caption(overview):
    count = uncertain_operations(overview)
    return f"{count} 项提交待确认" if count else ""
