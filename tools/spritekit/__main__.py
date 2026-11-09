import argparse
import json
import sys

from .air_text import action_text
from .palette_variants import PRESETS, write_variants
from .sheet_export import slice_sheet


def build_parser():
    parser = argparse.ArgumentParser(prog="python3 -m tools.spritekit")
    sub = parser.add_subparsers(dest="command", required=True)
    slice_cmd = sub.add_parser("slice")
    slice_cmd.add_argument("sheet")
    slice_cmd.add_argument("out")
    slice_cmd.add_argument("--mode", choices=["auto", "grid"], default="auto")
    slice_cmd.add_argument("--group", type=int, default=0)
    slice_cmd.add_argument("--start-item", type=int, default=0)
    slice_cmd.add_argument("--axis", default="feet", choices=["feet", "row-feet", "bottom-center", "center"])
    slice_cmd.add_argument("--background")
    slice_cmd.add_argument("--tolerance", type=int, default=8)
    slice_cmd.add_argument("--cell", type=int, nargs=2, metavar=("W", "H"))
    slice_cmd.add_argument("--grid", type=int, nargs=2, metavar=("COLS", "ROWS"))
    slice_cmd.add_argument("--rgba", action="store_true")
    palette_cmd = sub.add_parser("palettes")
    palette_cmd.add_argument("act")
    palette_cmd.add_argument("out")
    palette_cmd.add_argument("--only", nargs="+", choices=sorted(PRESETS))
    palette_cmd.add_argument("--keep", type=int, nargs="*", default=[])
    air_cmd = sub.add_parser("air")
    air_cmd.add_argument("manifest")
    air_cmd.add_argument("--action", type=int, required=True)
    air_cmd.add_argument("--ticks", type=int, default=6)
    air_cmd.add_argument("--attack", action="store_true")
    return parser


def run(args):
    if args.command == "slice":
        grid = None
        if args.mode == "grid":
            grid = {}
            if args.cell:
                grid["cell_width"], grid["cell_height"] = args.cell
            if args.grid:
                grid["columns"], grid["rows"] = args.grid
        manifest = slice_sheet(args.sheet, args.out, args.mode, args.group, args.start_item, args.axis,
                               args.background, args.tolerance, grid=grid, indexed=not args.rgba)
        print(json.dumps({"sprites": len(manifest["sprites"]), "exact_palette": manifest["exact_palette"]}))
    elif args.command == "palettes":
        for path in write_variants(args.act, args.out, args.only, args.keep):
            print(path)
    else:
        sys.stdout.write(action_text(args.manifest, args.action, args.ticks, args.attack))


run(build_parser().parse_args())
