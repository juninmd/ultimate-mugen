import json
import os
import shutil
import subprocess

from .convert import convert, feature_counts
from .dissect import hitdefs, summary

SCHEMA_CHECKER = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), "scripts", "umf-schema.mjs")


def _folder(path):
    if not os.path.isdir(path):
        raise ValueError(f"not a folder: {path}")
    return path


def run_convert(src, out, id, name, extra=None):
    fighter = convert(_folder(src), out, id, name, extra)
    report = open(os.path.join(out, "coverage.md"), encoding="utf-8").read().split("\n| state |")[0]
    return json.dumps(feature_counts(fighter), ensure_ascii=False) + "\n\n" + report.strip()


def run_inspect(src):
    return summary(_folder(src))


def run_hitdefs(file):
    if not os.path.isfile(file):
        raise ValueError(f"not a file: {file}")
    return hitdefs(file)


def run_coverage(out):
    return open(os.path.join(_folder(out), "coverage.md"), encoding="utf-8").read()


def run_validate(json_path):
    node = shutil.which("node")
    if not node or not os.path.isfile(SCHEMA_CHECKER):
        raise ValueError("schema check needs node and scripts/umf-schema.mjs from the ultimate-mugen repository")
    done = subprocess.run([node, SCHEMA_CHECKER, json_path], capture_output=True, text=True, timeout=120)
    if done.returncode:
        raise ValueError((done.stdout + done.stderr).strip())
    return done.stdout.strip()


TOOLS = {
    "convert": (run_convert, "Convert a M.U.G.E.N character folder into a UMF bundle (<id>.json, sprites/, coverage.md).", {
        "src": "M.U.G.E.N character folder (with the .def)", "out": "output folder", "id": "fighter id", "name": "display name",
        "extra": "optional JSON merged on top (extra moves, forms, helpers, profile, quotes)"}, ["src", "out", "id", "name"]),
    "inspect": (run_inspect, "Summarize a M.U.G.E.N character: SFF, SND, AIR actions, CMD commands, CNS/ST statedefs and controllers.", {
        "src": "M.U.G.E.N character folder"}, ["src"]),
    "hitdefs": (run_hitdefs, "List the HitDef values of the standard normals in a .cns or .st file.", {"file": ".cns or .st file"}, ["file"]),
    "coverage": (run_coverage, "Read the coverage.md of a converted bundle: what each state became and what was dropped.", {"out": "converted bundle folder"}, ["out"]),
    "validate": (run_validate, "Validate a UMF JSON against docs/schema/umf-2.schema.json.", {"json_path": "UMF fighter JSON"}, ["json_path"]),
}
