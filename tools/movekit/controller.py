def pair(value, default="0, 0"):
    if value is None:
        return default
    if isinstance(value, (list, tuple)):
        return ", ".join(str(v) for v in value)
    return str(value)


def as_list(value):
    if isinstance(value, list):
        return value
    return [value] if value else []


def controller(kind, label, **fields):
    lines = [f"[State 0, {label}]", f"type = {kind}"]
    for key, value in fields.items():
        key = key.replace("__", ".")
        if value is None:
            continue
        if isinstance(value, (list, tuple)):
            if key.startswith("trigger"):
                lines += [f"{key} = {v}" for v in value]
            else:
                lines.append(f"{key} = {pair(value)}")
        else:
            lines.append(f"{key} = {value}")
    return "\n".join(lines) + "\n\n"
