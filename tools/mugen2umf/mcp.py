import json
import sys

from . import __version__
from .tools import TOOLS

PROTOCOL = "2025-06-18"


def tool_list():
    return [{"name": name, "description": desc, "inputSchema": {"type": "object", "properties": {k: {"type": "string", "description": v} for k, v in props.items()}, "required": req, "additionalProperties": False}}
            for name, (_, desc, props, req) in TOOLS.items()]


def call(params):
    name, args = params.get("name"), params.get("arguments") or {}
    if name not in TOOLS:
        return {"content": [{"type": "text", "text": f"unknown tool: {name}"}], "isError": True}
    fn, _, props, req = TOOLS[name]
    bad = [k for k in args if k not in props] + [k for k in req if not isinstance(args.get(k), str)]
    if bad:
        return {"content": [{"type": "text", "text": f"invalid arguments: {', '.join(bad)}"}], "isError": True}
    try:
        return {"content": [{"type": "text", "text": fn(**args)}], "isError": False}
    except Exception as err:
        return {"content": [{"type": "text", "text": f"{type(err).__name__}: {err}"}], "isError": True}


def handle(msg):
    method, mid = msg.get("method"), msg.get("id")
    if mid is None:
        return None
    if method == "initialize":
        result = {"protocolVersion": msg.get("params", {}).get("protocolVersion", PROTOCOL), "capabilities": {"tools": {}}, "serverInfo": {"name": "mugen2umf", "version": __version__}}
    elif method == "ping":
        result = {}
    elif method == "tools/list":
        result = {"tools": tool_list()}
    elif method == "tools/call":
        result = call(msg.get("params") or {})
    else:
        return {"jsonrpc": "2.0", "id": mid, "error": {"code": -32601, "message": f"method not found: {method}"}}
    return {"jsonrpc": "2.0", "id": mid, "result": result}


def serve(stdin=sys.stdin, stdout=sys.stdout):
    for line in stdin:
        if not line.strip():
            continue
        try:
            reply = handle(json.loads(line))
        except json.JSONDecodeError:
            reply = {"jsonrpc": "2.0", "id": None, "error": {"code": -32700, "message": "parse error"}}
        if reply is not None:
            stdout.write(json.dumps(reply, ensure_ascii=False) + "\n")
            stdout.flush()
