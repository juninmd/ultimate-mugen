import json
import math
import struct


def load_glb(path):
    data = open(path, 'rb').read()
    if struct.unpack_from('<I', data, 0)[0] != 0x46546C67:
        raise ValueError(f'{path} is not a glb')
    at, doc, binary = 12, None, b''
    while at + 8 <= len(data):
        length, kind = struct.unpack_from('<II', data, at)
        body = data[at + 8:at + 8 + length]
        if kind == 0x4E4F534A:
            doc = json.loads(body)
        elif kind == 0x004E4942 and not binary:
            binary = bytes(body)
        at += 8 + ((length + 3) // 4) * 4
    return doc, binary


def save_glb(path, doc, binary):
    text = json.dumps(doc, separators=(',', ':')).encode()
    text += b' ' * ((4 - len(text) % 4) % 4)
    binary += b'\0' * ((4 - len(binary) % 4) % 4)
    total = 12 + 8 + len(text) + 8 + len(binary)
    with open(path, 'wb') as out:
        out.write(struct.pack('<III', 0x46546C67, 2, total))
        out.write(struct.pack('<II', len(text), 0x4E4F534A) + text)
        out.write(struct.pack('<II', len(binary), 0x004E4942) + binary)


SIZES = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}
FORMATS = {5120: 'b', 5121: 'B', 5122: 'h', 5123: 'H', 5125: 'I', 5126: 'f'}
LANES = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4, 'MAT4': 16}


def read_accessor(doc, binary, index):
    acc = doc['accessors'][index]
    view = doc['bufferViews'][acc['bufferView']]
    lanes, kind = LANES[acc['type']], acc['componentType']
    size = SIZES[kind]
    stride = view.get('byteStride', size * lanes)
    base = view.get('byteOffset', 0) + acc.get('byteOffset', 0)
    out = []
    for i in range(acc['count']):
        row = struct.unpack_from('<' + FORMATS[kind] * lanes, binary, base + i * stride)
        if acc.get('normalized') and kind == 5122:
            row = tuple(max(v / 32767.0, -1.0) for v in row)
        out.append(row)
    return out


def qmul(a, b):
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return (aw * bx + ax * bw + ay * bz - az * by,
            aw * by - ax * bz + ay * bw + az * bx,
            aw * bz + ax * by - ay * bx + az * bw,
            aw * bw - ax * bx - ay * by - az * bz)


def qinv(q):
    return (-q[0], -q[1], -q[2], q[3])


def qnorm(q):
    n = math.sqrt(sum(v * v for v in q)) or 1.0
    return tuple(v / n for v in q)


def qrot(q, v):
    p = qmul(qmul(q, (v[0], v[1], v[2], 0.0)), qinv(q))
    return (p[0], p[1], p[2])


def qslerp(a, b, t):
    dot = sum(x * y for x, y in zip(a, b))
    if dot < 0:
        b, dot = tuple(-v for v in b), -dot
    if dot > 0.9995:
        return qnorm(tuple(x + (y - x) * t for x, y in zip(a, b)))
    theta = math.acos(min(dot, 1.0))
    s = math.sin(theta)
    wa, wb = math.sin((1 - t) * theta) / s, math.sin(t * theta) / s
    return qnorm(tuple(x * wa + y * wb for x, y in zip(a, b)))


def from_to(u, v):
    nu = math.sqrt(sum(x * x for x in u)) or 1.0
    nv = math.sqrt(sum(x * x for x in v)) or 1.0
    u, v = [x / nu for x in u], [x / nv for x in v]
    d = sum(x * y for x, y in zip(u, v))
    if d < -0.999999:
        axis = (1.0, 0.0, 0.0) if abs(u[0]) < 0.9 else (0.0, 1.0, 0.0)
        c = (u[1] * axis[2] - u[2] * axis[1], u[2] * axis[0] - u[0] * axis[2], u[0] * axis[1] - u[1] * axis[0])
        return qnorm((c[0], c[1], c[2], 0.0))
    c = (u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0])
    return qnorm((c[0], c[1], c[2], 1.0 + d))


def mat_to_trs(m):
    t = (m[12], m[13], m[14])
    sx = math.sqrt(m[0] ** 2 + m[1] ** 2 + m[2] ** 2)
    sy = math.sqrt(m[4] ** 2 + m[5] ** 2 + m[6] ** 2)
    sz = math.sqrt(m[8] ** 2 + m[9] ** 2 + m[10] ** 2)
    r = [m[0] / sx, m[1] / sx, m[2] / sx, m[4] / sy, m[5] / sy, m[6] / sy, m[8] / sz, m[9] / sz, m[10] / sz]
    tr = r[0] + r[4] + r[8]
    if tr > 0:
        s = math.sqrt(tr + 1.0) * 2
        q = ((r[5] - r[7]) / s, (r[6] - r[2]) / s, (r[1] - r[3]) / s, 0.25 * s)
    elif r[0] > r[4] and r[0] > r[8]:
        s = math.sqrt(1.0 + r[0] - r[4] - r[8]) * 2
        q = (0.25 * s, (r[3] + r[1]) / s, (r[6] + r[2]) / s, (r[5] - r[7]) / s)
    elif r[4] > r[8]:
        s = math.sqrt(1.0 + r[4] - r[0] - r[8]) * 2
        q = ((r[3] + r[1]) / s, 0.25 * s, (r[7] + r[5]) / s, (r[6] - r[2]) / s)
    else:
        s = math.sqrt(1.0 + r[8] - r[0] - r[4]) * 2
        q = ((r[6] + r[2]) / s, (r[7] + r[5]) / s, 0.25 * s, (r[1] - r[3]) / s)
    return t, qnorm(q), (sx, sy, sz)


class Skeleton:
    def __init__(self, doc):
        self.doc = doc
        self.nodes = doc['nodes']
        self.parent = [None] * len(self.nodes)
        for i, n in enumerate(self.nodes):
            for c in n.get('children', []):
                self.parent[c] = i
        self.rest = [self.local_trs(i) for i in range(len(self.nodes))]

    def local_trs(self, i):
        n = self.nodes[i]
        if 'matrix' in n:
            return mat_to_trs(n['matrix'])
        return (tuple(n.get('translation', (0, 0, 0))), tuple(n.get('rotation', (0, 0, 0, 1))), tuple(n.get('scale', (1, 1, 1))))

    def world(self, local):
        out = [None] * len(self.nodes)

        def solve(i):
            if out[i] is not None:
                return out[i]
            t, r, s = local[i]
            p = self.parent[i]
            if p is None:
                out[i] = (t, r, s)
            else:
                pt, pr, ps = solve(p)
                scaled = (t[0] * ps[0], t[1] * ps[1], t[2] * ps[2])
                rt = qrot(pr, scaled)
                out[i] = ((pt[0] + rt[0], pt[1] + rt[1], pt[2] + rt[2]), qnorm(qmul(pr, r)), (ps[0] * s[0], ps[1] * s[1], ps[2] * s[2]))
            return out[i]

        for i in range(len(self.nodes)):
            solve(i)
        return out


KEEP_ATTRIBUTES = ('POSITION', 'TEXCOORD_0', 'JOINTS_0', 'WEIGHTS_0')


def compact(doc, binary):
    for mesh in doc.get('meshes', []):
        for prim in mesh['primitives']:
            prim['attributes'] = {k: v for k, v in prim['attributes'].items() if k in KEEP_ATTRIBUTES}
            prim.pop('targets', None)
    used = set()
    for mesh in doc.get('meshes', []):
        for prim in mesh['primitives']:
            used.update(prim['attributes'].values())
            if 'indices' in prim:
                used.add(prim['indices'])
    for skin in doc.get('skins', []):
        if 'inverseBindMatrices' in skin:
            used.add(skin['inverseBindMatrices'])
    for anim in doc.get('animations', []):
        for s in anim['samplers']:
            used.update((s['input'], s['output']))
    acc_map, accessors = {}, []
    for i, acc in enumerate(doc['accessors']):
        if i in used:
            acc_map[i] = len(accessors)
            accessors.append(acc)
    views_used = {a['bufferView'] for a in accessors} | {img['bufferView'] for img in doc.get('images', []) if 'bufferView' in img}
    view_map, views, out = {}, [], bytearray()
    for i, view in enumerate(doc['bufferViews']):
        if i not in views_used:
            continue
        while len(out) % 4:
            out.append(0)
        start = view.get('byteOffset', 0)
        chunk = binary[start:start + view['byteLength']]
        nv = dict(view)
        nv['byteOffset'] = len(out)
        out.extend(chunk)
        view_map[i] = len(views)
        views.append(nv)
    for acc in accessors:
        acc['bufferView'] = view_map[acc['bufferView']]
    for img in doc.get('images', []):
        if 'bufferView' in img:
            img['bufferView'] = view_map[img['bufferView']]
    for mesh in doc.get('meshes', []):
        for prim in mesh['primitives']:
            prim['attributes'] = {k: acc_map[v] for k, v in prim['attributes'].items()}
            if 'indices' in prim:
                prim['indices'] = acc_map[prim['indices']]
    for skin in doc.get('skins', []):
        if 'inverseBindMatrices' in skin:
            skin['inverseBindMatrices'] = acc_map[skin['inverseBindMatrices']]
    for anim in doc.get('animations', []):
        for s in anim['samplers']:
            s['input'], s['output'] = acc_map[s['input']], acc_map[s['output']]
    doc['accessors'], doc['bufferViews'] = accessors, views
    doc['buffers'] = [{'byteLength': len(out)}]
    return doc, bytes(out)
