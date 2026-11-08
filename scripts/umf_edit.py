import json
def load(path): return json.load(open(path, encoding="utf8"))
def style(path):
    raw = open(path, encoding="utf8", newline="").read()
    eol = "\r\n" if "\r\n" in raw[-3:] else "\n" if raw.endswith("\n") else ""
    return raw, eol, (2 if raw.startswith("{\n") or raw.startswith("{\r\n") else None)
def save(path, d):
    raw, eol, indent = style(path)
    text = json.dumps(d, ensure_ascii=False, indent=indent)
    if indent: text = text.replace("\n", "\r\n" if "\r\n" in raw else "\n")
    open(path, "w", encoding="utf8", newline="").write(text + eol)
