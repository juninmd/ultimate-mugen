import json
import tempfile
import unittest
from pathlib import Path

import numpy as np
from PIL import Image

from .air_text import action_text
from .collision_boxes import anatomical_boxes, attack_boxes, interpolate_boxes
from .palette_variants import load_act, save_act, variant_colors, write_variants
from .regions import label_components
from .sheet_export import slice_sheet


def synthetic_sheet(path):
    image = Image.new("RGB", (120, 60), (255, 0, 255))
    pixels = image.load()
    for x0 in (10, 70):
        for x in range(x0, x0 + 20):
            for y in range(8, 56):
                pixels[x, y] = (200, 40 + x0, 30)
    image.save(path)


class SpriteKitTests(unittest.TestCase):
    def test_labels_separate_blobs(self):
        mask = np.zeros((10, 10), bool)
        mask[1:3, 1:3] = True
        mask[6:9, 6:9] = True
        self.assertEqual(len(set(label_components(mask).ravel().tolist()) - {0}), 2)

    def test_slice_auto_exports_indexed_pngs_with_feet_axis(self):
        with tempfile.TemporaryDirectory() as tmp:
            sheet = Path(tmp) / "sheet.png"
            synthetic_sheet(sheet)
            manifest = slice_sheet(sheet, Path(tmp) / "out", group=9000)
            self.assertEqual([s["item"] for s in manifest["sprites"]], [0, 1])
            self.assertTrue(manifest["exact_palette"])
            first = manifest["sprites"][0]
            self.assertEqual(first["axis"], [10, 47])
            with Image.open(Path(tmp) / "out" / first["file"]) as exported:
                self.assertEqual(exported.mode, "P")

    def test_slice_grid(self):
        with tempfile.TemporaryDirectory() as tmp:
            sheet = Path(tmp) / "sheet.png"
            synthetic_sheet(sheet)
            manifest = slice_sheet(sheet, Path(tmp) / "out", mode="grid", grid={"columns": 2, "rows": 1})
            self.assertEqual(len(manifest["sprites"]), 2)

    def test_palette_variant_keeps_index_zero_and_round_trips_act(self):
        colors = [(255, 0, 255), (200, 40, 30)] + [(0, 0, 0)] * 254
        shifted = variant_colors(colors, hue_shift=120)
        self.assertEqual(shifted[0], colors[0])
        self.assertNotEqual(shifted[1], colors[1])
        with tempfile.TemporaryDirectory() as tmp:
            act = Path(tmp) / "base.act"
            save_act(colors, act)
            self.assertEqual(load_act(act), colors)
            self.assertEqual(len(write_variants(act, Path(tmp) / "v", ["red", "gray"])), 2)

    def test_boxes_are_relative_to_axis(self):
        mask = np.zeros((80, 60), bool)
        mask[10:78, 20:40] = True
        boxes = anatomical_boxes(mask, (30, 77))
        self.assertEqual(len(boxes), 3)
        self.assertTrue(all(x0 < x1 and y0 < y1 for x0, y0, x1, y1 in boxes))
        self.assertEqual(attack_boxes(mask, (30, 77)), [])
        mask[40:44, 40:58] = True
        self.assertEqual(len(attack_boxes(mask, (30, 77))), 1)
        self.assertEqual(len(interpolate_boxes(boxes, boxes, 2)), 2)

    def test_air_text_lists_frames_and_boxes(self):
        with tempfile.TemporaryDirectory() as tmp:
            sheet = Path(tmp) / "sheet.png"
            synthetic_sheet(sheet)
            slice_sheet(sheet, Path(tmp) / "out", group=9000)
            text = action_text(Path(tmp) / "out" / "manifest.json", 200, ticks=4)
            self.assertTrue(text.startswith("[Begin Action 200]"))
            self.assertIn("Clsn2:", text)
            self.assertIn("9000, 1, 0, 0, 4", text)
            self.assertEqual(json.loads((Path(tmp) / "out" / "manifest.json").read_text())["sprites"][0]["file"], "9000_0.png")


if __name__ == "__main__":
    unittest.main()
