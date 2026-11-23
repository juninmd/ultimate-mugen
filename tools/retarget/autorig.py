import json
import math
import struct
import sys
from pathlib import Path

from gltf_math import Skeleton, load_glb, qinv, qrot, read_accessor, save_glb

CHAINS = [
    ('hips', 'spine1'), ('spine1', 'spine2'), ('spine2', 'spine3'), ('spine3', 'neck'), ('neck', 'head'), ('head', '+head'),
    ('shoulderL', 'upperArmL'), ('upperArmL', 'forearmL'), ('forearmL', 'handL'), ('handL', '+handL'),
    ('shoulderR', 'upperArmR'), ('upperArmR', 'forearmR'), ('forearmR', 'handR'), ('handR', '+handR'),
    ('thighL', 'shinL'), ('shinL', 'footL'), ('footL', 'toeL'), ('toeL', '+toeL'),
    ('thighR', 'shinR'), ('shinR', 'footR'), ('footR', 'toeR'), ('toeR', '+toeR'),
]
ARM = ['shoulder', 'upperArm', 'forearm', 'hand', 'finger']


def sub(a, b):
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def add(a, b):
    return (a[0] + b[0], a[1] + b[1], a[2] + b[2])


def mul(a, k):
    return (a[0] * k, a[1] * k, a[2] * k)


def seg_dist(p, a, b):
    ab, ap = sub(b, a), sub(p, a)
    den = sum(x * x for x in ab) or 1e-12
    t = max(0.0, min(1.0, sum(x * y for x, y in zip(ab, ap)) / den))
    q = add(a, mul(ab, t))
    return math.sqrt(sum(x * x for x in sub(p, q)))


def mesh_vertices(doc, binary):
    sk = Skeleton(doc)
    world = sk.world(sk.rest)
    out = []
    for i, n in enumerate(doc['nodes']):
        if 'mesh' not in n:
            continue
        t, r, s = world[i]
        for prim in doc['meshes'][n['mesh']]['primitives']:
            pos = read_accessor(doc, binary, prim['attributes']['POSITION'])
            pos = [add(t, qrot(r, (p[0] * s[0], p[1] * s[1], p[2] * s[2]))) for p in pos]
            uv = read_accessor(doc, binary, prim['attributes']['TEXCOORD_0']) if 'TEXCOORD_0' in prim['attributes'] else [(0.0, 0.0)] * len(pos)
            idx = [v[0] for v in read_accessor(doc, binary, prim['indices'])] if 'indices' in prim else list(range(len(pos)))
            out.append({'pos': pos, 'uv': uv, 'idx': idx, 'material': prim.get('material')})
    return out


def fit_skeleton(src, names, verts):
    sk = Skeleton(src.doc if hasattr(src, 'doc') else src)
    rest = sk.world(sk.rest)
    node = {k: sk.nodes.index(next(n for n in sk.nodes if n.get('name') == v)) for k, v in names.items()}
    ys = sorted(p[1] for v in verts for p in v['pos'])
    lo, hi = ys[0], ys[int(len(ys) * 0.995) - 1]
    head_top = rest[node['head']][0][1] + 0.18
    scale = (hi - lo) / head_top
    xs = [p for v in verts for p in v['pos']]
    reach = max(abs(p[0]) for p in xs)
    hand_reach = (rest[node['handL']][0][0] + 0.12) * scale
    arm_k = max(0.7, min(1.6, reach / max(hand_reach, 1e-6)))
    local = [list(x) for x in sk.rest]
    for i, (t, r, s) in enumerate(sk.rest):
        local[i][0] = mul(t, scale) if sk.parent[i] is not None else t
    for side in 'LR':
        for part in ARM[1:]:
            i = node[part + side]
            local[i][0] = mul(local[i][0], arm_k)
    sk.rest = [tuple(x) for x in local]
    return sk, node, lo, scale


def build(dst_glb, ual_glb, names, out_path):
    doc, binary = load_glb(dst_glb)
    verts = mesh_vertices(doc, binary)
    src_doc, _ = load_glb(ual_glb)
    sk, node, floor, scale = fit_skeleton(src_doc, names, verts)
    world = sk.world(sk.rest)
    cx = sum(p[0] for v in verts for p in v['pos']) / sum(len(v['pos']) for v in verts)
    cz = sum(p[2] for v in verts for p in v['pos']) / sum(len(v['pos']) for v in verts)
    hips = world[node['hips']][0]
    shift = (hips[0] - cx, -floor, hips[2] - cz)
    for v in verts:
        v['pos'] = [add(p, shift) for p in v['pos']]
    joints = src_doc['skins'][0]['joints']
    slot = {j: k for k, j in enumerate(joints)}

    def point(name):
        if name.startswith('+'):
            base = name[1:]
            i = node[base]
            p = world[sk.parent[i]][0]
            d = sub(world[i][0], p)
            return add(world[i][0], mul(d, 0.6))
        return world[node[name]][0]

    segs = [(slot[node[a]], point(a), point(b), a[-1] if a[-1] in 'LR' else '') for a, b in CHAINS]
    side_gap = 0.03 * scale
    for v in verts:
        jw = []
        for p in v['pos']:
            ranked = []
            for j, a, b, side in segs:
                if side == 'L' and p[0] < -side_gap or side == 'R' and p[0] > side_gap:
                    continue
                d = seg_dist(p, a, b) + 1e-4 * scale
                ranked.append((1.0 / d ** 4, j))
            ranked.sort(reverse=True)
            top = ranked[:3] + [(0.0, 0)] * (3 - len(ranked[:3]))
            total = sum(w for w, _ in top) or 1.0
            jw.append(([top[0][1], top[1][1], top[2][1], 0], [top[0][0] / total, top[1][0] / total, top[2][0] / total, 0.0]))
        v['jw'] = jw
    write(doc, binary, src_doc, sk, world, joints, verts, out_path)


def inverse_trs(t, r, s):
    ri = qinv(r)
    cols = [qrot(ri, e) for e in ((1, 0, 0), (0, 1, 0), (0, 0, 1))]
    k = 1.0 / s[0]
    m = [cols[0][0] * k, cols[0][1] * k, cols[0][2] * k, 0, cols[1][0] * k, cols[1][1] * k, cols[1][2] * k, 0, cols[2][0] * k, cols[2][1] * k, cols[2][2] * k, 0, 0, 0, 0, 1]
    it = qrot(ri, t)
    m[12], m[13], m[14] = -it[0] * k, -it[1] * k, -it[2] * k
    return m


def write(doc, binary, src_doc, sk, world, joints, verts, out_path):
    blob = bytearray()
    views, accessors = [], []

    def push(data, kind, comp, count, target=None, extra=None):
        while len(blob) % 4:
            blob.append(0)
        start = len(blob)
        blob.extend(data)
        view = {'buffer': 0, 'byteOffset': start, 'byteLength': len(data)}
        if target:
            view['target'] = target
        views.append(view)
        acc = {'bufferView': len(views) - 1, 'componentType': comp, 'count': count, 'type': kind}
        acc.update(extra or {})
        accessors.append(acc)
        return len(accessors) - 1

    images = []
    for img in doc.get('images', []):
        v = doc['bufferViews'][img['bufferView']]
        data = binary[v.get('byteOffset', 0):v.get('byteOffset', 0) + v['byteLength']]
        while len(blob) % 4:
            blob.append(0)
        views.append({'buffer': 0, 'byteOffset': len(blob), 'byteLength': len(data)})
        blob.extend(data)
        images.append({'mimeType': img.get('mimeType', 'image/png'), 'bufferView': len(views) - 1})
    prims = []
    for v in verts:
        n = len(v['pos'])
        lo = [min(p[k] for p in v['pos']) for k in range(3)]
        hi = [max(p[k] for p in v['pos']) for k in range(3)]
        pos = push(b''.join(struct.pack('<3f', *p) for p in v['pos']), 'VEC3', 5126, n, 34962, {'min': lo, 'max': hi})
        uv = push(b''.join(struct.pack('<2f', *u[:2]) for u in v['uv']), 'VEC2', 5126, n, 34962)
        jo = push(b''.join(struct.pack('<4H', *j) for j, _ in v['jw']), 'VEC4', 5123, n, 34962)
        we = push(b''.join(struct.pack('<4f', *w) for _, w in v['jw']), 'VEC4', 5126, n, 34962)
        ix = push(b''.join(struct.pack('<I', i) for i in v['idx']), 'SCALAR', 5125, len(v['idx']), 34963)
        prim = {'attributes': {'POSITION': pos, 'TEXCOORD_0': uv, 'JOINTS_0': jo, 'WEIGHTS_0': we}, 'indices': ix}
        if v['material'] is not None:
            prim['material'] = v['material']
        prims.append(prim)
    ibm = push(b''.join(struct.pack('<16f', *inverse_trs(*world[j])) for j in joints), 'MAT4', 5126, len(joints))
    nodes = []
    for i, n in enumerate(sk.nodes):
        t, r, s = sk.rest[i]
        entry = {'name': n.get('name', f'n{i}'), 'translation': list(t), 'rotation': list(r), 'scale': list(s)}
        if n.get('children'):
            entry['children'] = list(n['children'])
        nodes.append(entry)
    roots = [i for i, p in enumerate(sk.parent) if p is None]
    nodes.append({'name': 'hollow_mesh', 'mesh': 0, 'skin': 0})
    out = {
        'asset': {'version': '2.0', 'generator': 'ultimate-mugen autorig', 'extras': doc['asset'].get('extras', {})},
        'scene': 0, 'scenes': [{'nodes': roots + [len(nodes) - 1]}], 'nodes': nodes,
        'meshes': [{'name': 'hollow', 'primitives': prims}],
        'skins': [{'joints': joints, 'inverseBindMatrices': ibm}],
        'materials': doc.get('materials', []), 'textures': doc.get('textures', []), 'samplers': doc.get('samplers', []),
        'images': images, 'accessors': accessors, 'bufferViews': views, 'buffers': [{'byteLength': len(blob)}],
    }
    save_glb(out_path, out, bytes(blob))


def main():
    dst_glb, ual_dir, out_path = sys.argv[1:4]
    here = Path(__file__).parent
    rigs = json.loads((here / 'rigs.json').read_text())
    config = json.loads((here / 'clips.json').read_text())
    build(dst_glb, str(Path(ual_dir) / config['sources']['ual1']), rigs['sources']['ual1'], out_path)
    print(f'{out_path}: rigged')


if __name__ == '__main__':
    main()
