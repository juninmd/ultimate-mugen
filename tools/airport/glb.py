import json
import struct


class Mesh:
    def __init__(self):
        self.groups = {}

    def tri(self, mat, a, b, c):
        verts, idx = self.groups.setdefault(mat, ([], []))
        base = len(verts)
        verts.extend([a, b, c])
        idx.extend([base, base + 1, base + 2])

    def quad(self, mat, a, b, c, d):
        self.tri(mat, a, b, c)
        self.tri(mat, a, c, d)

    def merge(self, other, fn):
        for mat, (verts, idx) in other.groups.items():
            for i in range(0, len(idx), 3):
                self.tri(mat, *(fn(verts[j]) for j in idx[i : i + 3]))


class Writer:
    def __init__(self, materials):
        self.materials = list(materials)
        self.blob = bytearray()
        self.views, self.accessors, self.meshes, self.nodes = [], [], [], []
        self.animations = []

    def _view(self, data):
        while len(self.blob) % 4:
            self.blob.append(0)
        self.views.append({"buffer": 0, "byteOffset": len(self.blob), "byteLength": len(data)})
        self.blob.extend(data)
        return len(self.views) - 1

    def _accessor(self, data, ctype, kind, count, extra=None):
        entry = {"bufferView": self._view(data), "componentType": ctype, "count": count, "type": kind}
        entry.update(extra or {})
        self.accessors.append(entry)
        return len(self.accessors) - 1

    def add_node(self, name, mesh=None, translation=None, parent=None):
        node = {"name": name}
        if translation:
            node["translation"] = list(translation)
        if mesh is not None and mesh.groups:
            node["mesh"] = self._mesh(mesh)
        self.nodes.append(node)
        index = len(self.nodes) - 1
        if parent is not None:
            self.nodes[parent].setdefault("children", []).append(index)
        return index

    def _mesh(self, mesh):
        prims = []
        for mat, (raw, raw_idx) in mesh.groups.items():
            seen, verts, idx = {}, [], []
            for j in raw_idx:
                key = tuple(round(c, 3) for c in raw[j])
                if key not in seen:
                    seen[key] = len(verts)
                    verts.append(key)
                idx.append(seen[key])
            assert len(verts) < 65536, mat
            flat = [c for v in verts for c in v]
            xs, ys, zs = flat[0::3], flat[1::3], flat[2::3]
            pos = self._accessor(
                struct.pack(f"<{len(flat)}f", *flat), 5126, "VEC3", len(verts),
                {"min": [min(xs), min(ys), min(zs)], "max": [max(xs), max(ys), max(zs)]},
            )
            ids = self._accessor(struct.pack(f"<{len(idx)}H", *idx), 5123, "SCALAR", len(idx))
            prims.append({"attributes": {"POSITION": pos}, "indices": ids, "material": self.materials.index(mat)})
        self.meshes.append({"primitives": prims})
        return len(self.meshes) - 1

    def animate(self, name, tracks):
        samplers, channels = [], []
        for node, path, times, values in tracks:
            n = len(values[0])
            t = self._accessor(struct.pack(f"<{len(times)}f", *times), 5126, "SCALAR", len(times),
                               {"min": [min(times)], "max": [max(times)]})
            flat = [c for v in values for c in v]
            v = self._accessor(struct.pack(f"<{len(flat)}f", *flat), 5126, "VEC3" if n == 3 else "VEC4", len(values))
            samplers.append({"input": t, "output": v, "interpolation": "LINEAR"})
            channels.append({"sampler": len(samplers) - 1, "target": {"node": node, "path": path}})
        self.animations.append({"name": name, "samplers": samplers, "channels": channels})

    def save(self, path, materials_def):
        mats = []
        for name in self.materials:
            color, glow = materials_def[name]
            m = {"name": name, "pbrMetallicRoughness": {"baseColorFactor": list(color) + [1.0]}}
            if glow:
                m["emissiveFactor"] = [1.0, 1.0, 1.0]
            mats.append(m)
        doc = {
            "asset": {"version": "2.0", "generator": "tools/airport"},
            "scene": 0,
            "scenes": [{"nodes": [i for i, n in enumerate(self.nodes) if not self._has_parent(i)]}],
            "nodes": self.nodes, "meshes": self.meshes, "materials": mats,
            "accessors": self.accessors, "bufferViews": self.views,
            "buffers": [{"byteLength": len(self.blob)}],
        }
        if self.animations:
            doc["animations"] = self.animations
        js = json.dumps(doc, separators=(",", ":")).encode()
        js += b" " * (-len(js) % 4)
        blob = bytes(self.blob) + b"\0" * (-len(self.blob) % 4)
        total = 12 + 8 + len(js) + 8 + len(blob)
        with open(path, "wb") as f:
            f.write(struct.pack("<III", 0x46546C67, 2, total))
            f.write(struct.pack("<II", len(js), 0x4E4F534A) + js)
            f.write(struct.pack("<II", len(blob), 0x004E4942) + blob)

    def _has_parent(self, i):
        return any(i in n.get("children", []) for n in self.nodes)
