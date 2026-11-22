import argparse
import sys

from . import __version__
from .tools import TOOLS


def main(argv=None):
    ap = argparse.ArgumentParser(prog="mugen2umf", description="M.U.G.E.N to UMF converter (CLI and MCP server).")
    ap.add_argument("--version", action="version", version=__version__)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name, (_, desc, props, req) in TOOLS.items():
        p = sub.add_parser(name, help=desc, description=desc)
        for key in props:
            if key in req and (name != "convert" or key in ("src", "out")):
                p.add_argument(key, help=props[key])
            else:
                p.add_argument(f"--{key}", required=key in req, help=props[key])
    sub.add_parser("mcp", help="Run as an MCP server over stdio.")
    a = ap.parse_args(argv)
    if a.cmd == "mcp":
        from .mcp import serve
        serve()
        return 0
    fn, _, props, _ = TOOLS[a.cmd]
    try:
        print(fn(**{k: getattr(a, k) for k in props if getattr(a, k) is not None}))
    except ValueError as err:
        print(f"error: {err}", file=sys.stderr)
        return 1
    return 0
