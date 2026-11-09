import argparse

from .build import build_from_spec


def run():
    parser = argparse.ArgumentParser(prog="python3 -m tools.movekit")
    parser.add_argument("spec")
    parser.add_argument("out")
    args = parser.parse_args()
    result = build_from_spec(args.spec, args.out)
    print(f"actions: {result['air'].count('[Begin Action')}, states: {result['cns'].count('[Statedef')}, commands: {result['cmd'].count('[Command]')}")


run()
