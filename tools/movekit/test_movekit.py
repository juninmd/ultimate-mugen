import re
import tempfile
import unittest
from pathlib import Path

from PIL import Image

from tools.spritekit.sheet_export import slice_sheet

from .build import build_from_spec, build_moves
from .animation import hit_frames
from .phases import expand_phases
from .sprites import SpriteLibrary

EXAMPLE = Path(__file__).parent / "examples" / "moves.toml"


def make_sprites(root):
    sheet = root / "sheet.png"
    image = Image.new("RGB", (160, 70), (255, 0, 255))
    pixels = image.load()
    for x in range(10, 30):
        for y in range(10, 66):
            pixels[x, y] = (200, 40, 30)
    for x in range(80, 120):
        for y in range(10, 66):
            pixels[x, y] = (200, 90, 30)
    image.save(sheet)
    slice_sheet(sheet, root / "sprites", group=9000)


class MoveKitTests(unittest.TestCase):
    def build_example(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        root = Path(tmp.name)
        make_sprites(root)
        spec = root / "moves.toml"
        spec.write_text(EXAMPLE.read_text(encoding="utf-8"), encoding="utf-8")
        return build_from_spec(spec, root / "out"), root

    def test_outputs_are_written_and_consistent(self):
        result, root = self.build_example()
        for name in ("moves.air", "moves.cns", "moves.cmd", "statedef-1.cns"):
            self.assertTrue((root / "out" / name).read_text(encoding="utf-8").strip())
        states = {int(n) for n in re.findall(r"\[Statedef (\d+)\]", result["cns"])}
        self.assertEqual(states, {200, 1000, 3000, 3001})
        self.assertEqual(result["air"].count("[Begin Action"), 4)
        self.assertIn("Clsn2:", result["air"])

    def test_hitdef_uses_the_frame_with_clsn1(self):
        result, _ = self.build_example()
        block = result["cns"].split("[Statedef 200]")[1].split("[Statedef 1000]")[0]
        self.assertIn("type = HitDef", block)
        self.assertIn("trigger1 = animelem = 2", block)
        self.assertEqual(hit_frames("Clsn1: 1\n  Clsn1[0] = 0,0,1,1\n0,0,0,0,3"), [1])

    def test_super_has_pause_cost_and_phase_chain(self):
        result, _ = self.build_example()
        self.assertIn("type = SuperPause", result["cns"])
        self.assertIn("poweradd = -3000", result["cns"])
        self.assertIn("value = 3001", result["cns"])

    def test_entry_block_orders_super_before_normals(self):
        result, _ = self.build_example()
        entry = result["entry"]
        self.assertLess(entry.index("spec_3000"), entry.index("spec_200"))
        self.assertIn("power >= 3000", entry)
        self.assertIn('name = "spec_1000"', result["cmd"])

    def test_invalid_specs_are_rejected(self):
        library = SpriteLibrary()
        with self.assertRaises(ValueError):
            build_moves([{"state": 1, "kind": "bogus"}], {}, library)
        with self.assertRaises(ValueError):
            build_moves([{"state": 1}, {"state": 1}], {}, library)
        with self.assertRaises(ValueError):
            expand_phases({"state": 5, "phases": [{"like": "missing"}]})


if __name__ == "__main__":
    unittest.main()
