import json
from pathlib import Path

from tools.spritekit.air_text import sprite_mask


class SpriteLibrary:
    def __init__(self, manifest_paths=()):
        self.entries = {}
        for path in manifest_paths:
            base = Path(path).parent
            for entry in json.loads(Path(path).read_text(encoding="utf-8"))["sprites"]:
                self.entries[(entry["group"], entry["item"])] = (base / entry["file"], tuple(entry["axis"]))

    def mask_and_axis(self, group, item):
        found = self.entries.get((group, item))
        if found is None:
            return None
        return sprite_mask(found[0]), found[1]
