import json
import math
import struct
import sys
from pathlib import Path

from clips import ClipSource
from gltf_math import Skeleton, compact, from_to, load_glb, qinv, qmul, qnorm, qrot, qslerp, save_glb

FPS = 20
HORIZONTAL_LIMIT = 0.12
IDENTITY = (0.0, 0.0, 0.0, 1.0)


def depth(skeleton, i):
    d = 0
    while skeleton.parent[i] is not None:
        i, d = skeleton.parent[i], d + 1
    return d


class Retarget:
    def __init__(self, dst_doc, rig, names):
        self.dst = Skeleton(dst_doc)
        self.rest = self.dst.world(self.dst.rest)
        self.rig = rig
        self.names = names
        by_name = {n.get('name'): i for i, n in enumerate(dst_doc['nodes'])}
        ix = lambda v: v if isinstance(v, int) else by_name[v] if v in by_name else int(v)
        self.root = ix(rig['root'])
        self.delta = {ix(k): v for k, v in rig['delta'].items()}
        self.direction = {ix(d): (ix(c), sb, sc) for d, c, sb, sc in rig['direction']}
        self.follow = {ix(f): (ix(lead), k) for f, lead, k in rig.get('follow', [])}
        self.relative = {ix(d): (ix(p), ix(gp), sb, sp, sgp) for d, p, gp, sb, sp, sgp in rig.get('relative', [])}
        self.driven = sorted(set(self.delta) | set(self.direction) | set(self.follow) | set(self.relative), key=lambda i: depth(self.dst, i))

    def frame(self, src, sworld, srest):
        def node(name):
            return src.by_name[self.names[name]]

        def pos(w, i):
            return w[i][0]

        def sub(a, b):
            return (a[0] - b[0], a[1] - b[1], a[2] - b[2])

        target = {}
        for d, bone in self.delta.items():
            i = node(bone)
            delta = qmul(sworld[i][1], qinv(srest[i][1]))
            target[d] = qnorm(qmul(delta, self.rest[d][1]))
        for d, (child, sbone, schild) in self.direction.items():
            rest_dir = sub(pos(self.rest, child), pos(self.rest, d))
            now_dir = sub(pos(sworld, node(schild)), pos(sworld, node(sbone)))
            target[d] = qnorm(qmul(from_to(rest_dir, now_dir), self.rest[d][1]))
        world_rot = {}

        def rot(i):
            if i in world_rot:
                return world_rot[i]
            p = self.dst.parent[i]
            parent = rot(p) if p is not None else IDENTITY
            if i in target:
                world_rot[i] = target[i]
            else:
                world_rot[i] = qnorm(qmul(parent, self.dst.rest[i][1]))
            return world_rot[i]

        local = {}
        for i in self.driven:
            if i in self.relative:
                p, gp, sbone, sparent, sgrand = self.relative[i]
                s_parent_delta = qmul(sworld[node(sparent)][1], qinv(srest[node(sparent)][1]))
                s_delta = qmul(sworld[node(sbone)][1], qinv(srest[node(sbone)][1]))
                extra = qmul(qinv(s_parent_delta), s_delta)
                align = from_to(sub(pos(srest, node(sparent)), pos(srest, node(sgrand))), sub(pos(self.rest, p), pos(self.rest, gp)))
                extra = qmul(qmul(align, extra), qinv(align))
                d_parent_delta = qmul(rot(p), qinv(self.rest[p][1]))
                target[i] = qnorm(qmul(qmul(d_parent_delta, extra), self.rest[i][1]))
            if i in self.follow:
                lead, k = self.follow[i]
                lead_delta = qmul(rot(lead), qinv(self.rest[lead][1]))
                target[i] = qnorm(qmul(qslerp(IDENTITY, lead_delta, k), self.rest[i][1]))
            p = self.dst.parent[i]
            local[i] = qnorm(qmul(qinv(rot(p)), target[i]))
            world_rot.pop(i, None)
            rot(i)
        hips = node('hips')
        move = sub(pos(sworld, hips), pos(srest, hips))
        scale = self.rest[self.root][0][1] / max(srest[hips][0][1], 1e-6)
        limit = HORIZONTAL_LIMIT * self.rest[self.root][0][1]
        move = (clamp(move[0] * scale, limit), move[1] * scale, clamp(move[2] * scale, limit))
        parent = self.dst.parent[self.root]
        pt, pr, ps = self.rest[parent]
        rest_t = self.dst.rest[self.root][0]
        world_move = qrot(qinv(pr), move)
        translation = tuple(rest_t[k] + world_move[k] / ps[k] for k in range(3))
        return local, translation


def clamp(v, limit):
    return max(-limit, min(limit, v))


class Writer:
    def __init__(self, doc, binary):
        self.doc, self.binary = doc, bytearray(binary)
        doc.setdefault('bufferViews', [])
        doc.setdefault('accessors', [])
        doc['animations'] = []

    def accessor(self, rows, kind, extra=None, quantized=False):
        while len(self.binary) % 4:
            self.binary.append(0)
        lanes = len(rows[0])
        start = len(self.binary)
        for r in rows:
            if quantized:
                self.binary += struct.pack('<' + 'h' * lanes, *(max(-32767, min(32767, round(v * 32767))) for v in r))
            else:
                self.binary += struct.pack('<' + 'f' * lanes, *r)
        self.doc['bufferViews'].append({'buffer': 0, 'byteOffset': start, 'byteLength': len(self.binary) - start})
        acc = {'bufferView': len(self.doc['bufferViews']) - 1, 'componentType': 5122 if quantized else 5126, 'count': len(rows), 'type': kind}
        if quantized:
            acc['normalized'] = True
        acc.update(extra or {})
        self.doc['accessors'].append(acc)
        return len(self.doc['accessors']) - 1

    def clip(self, name, times, rotations, translation_node, translations):
        samplers, channels = [], []
        t = self.accessor([(x,) for x in times], 'SCALAR', {'min': [times[0]], 'max': [times[-1]]})
        for node, rows in rotations.items():
            samplers.append({'input': t, 'output': self.accessor(rows, 'VEC4', quantized=True), 'interpolation': 'LINEAR'})
            channels.append({'sampler': len(samplers) - 1, 'target': {'node': node, 'path': 'rotation'}})
        samplers.append({'input': t, 'output': self.accessor(translations, 'VEC3'), 'interpolation': 'LINEAR'})
        channels.append({'sampler': len(samplers) - 1, 'target': {'node': translation_node, 'path': 'translation'}})
        self.doc['animations'].append({'name': name, 'samplers': samplers, 'channels': channels})

    def finish(self):
        self.doc['buffers'][0]['byteLength'] = len(self.binary)
        return self.doc, bytes(self.binary)


def bake(sources, rigs, config, dst_doc, dst_bin, rig_name):
    writer = Writer(dst_doc, dst_bin)
    for out_name, family, clip in config['clips']:
        src = sources[family]
        target = Retarget(dst_doc, rigs[rig_name], rigs['sources'][family])
        srest = src.skeleton.world(src.skeleton.rest)
        duration = src.duration(clip)
        count = max(2, int(math.ceil(duration * FPS)) + 1)
        times, rotations, translations = [], {i: [] for i in target.driven}, []
        for f in range(count):
            t = min(f / FPS, duration)
            local, translation = target.frame(src, src.skeleton.world(src.local_pose(clip, t)), srest)
            times.append(t)
            for i in target.driven:
                rotations[i].append(local[i])
            translations.append(translation)
        writer.clip(out_name, times, rotations, target.root, translations)
    return writer.finish()


def main():
    ual_dir, dst_in, dst_out, rig_name = sys.argv[1:5]
    here = Path(__file__).parent
    config = json.loads((here / 'clips.json').read_text())
    rigs = json.loads((here / 'rigs.json').read_text())
    sources = {k: ClipSource(str(Path(ual_dir) / v)) for k, v in config['sources'].items()}
    doc, binary = load_glb(dst_in)
    doc, binary = compact(*bake(sources, rigs, config, doc, binary, rig_name))
    save_glb(dst_out, doc, binary)
    print(f'{dst_out}: {len(doc["animations"])} clips')


if __name__ == '__main__':
    main()
