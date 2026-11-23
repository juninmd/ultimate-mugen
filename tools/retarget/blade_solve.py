import itertools
import math

from gltf_math import Skeleton, qrot, read_accessor


def blade_axis(doc, binary, sword):
    skin = doc['skins'][0]
    slot = skin['joints'].index(sword)
    ibm = read_accessor(doc, binary, skin['inverseBindMatrices'])[slot]
    best, tip = 0.0, (0.0, 1.0, 0.0)
    for mesh in doc['meshes']:
        for prim in mesh['primitives']:
            attrs = prim['attributes']
            if 'JOINTS_0' not in attrs:
                continue
            pos = read_accessor(doc, binary, attrs['POSITION'])
            joints = read_accessor(doc, binary, attrs['JOINTS_0'])
            weights = read_accessor(doc, binary, attrs['WEIGHTS_0'])
            for p, j, w in zip(pos, joints, weights):
                if j[max(range(4), key=lambda k: w[k])] != slot:
                    continue
                local = tuple(ibm[r] * p[0] + ibm[4 + r] * p[1] + ibm[8 + r] * p[2] + ibm[12 + r] for r in range(3))
                length = math.sqrt(sum(v * v for v in local))
                if length > best:
                    best, tip = length, tuple(v / length for v in local)
    return tip


class BladeSolver:
    def __init__(self, doc, binary, target, src, sword_name):
        by_name = {n.get('name'): i for i, n in enumerate(doc['nodes'])}
        self.sword = by_name.get(sword_name)
        self.skeleton = Skeleton(doc)
        self.target, self.src = target, src
        self.axis = blade_axis(doc, binary, self.sword) if self.sword is not None else None

    def direction(self, spec, pose_type):
        local, translation = self.target.frame(self.src, self.src.frame(pose_type(spec)), self.src.rest)
        pose = list(self.skeleton.rest)
        for i, q in local.items():
            pose[i] = (pose[i][0], q, pose[i][2])
        root = self.target.root
        pose[root] = (translation, pose[root][1], pose[root][2])
        return qrot(self.skeleton.world(pose)[self.sword][1], self.axis)

    def solve(self, spec, want, pose_type):
        if self.sword is None:
            return spec.get('wristR', [0, 0, 0])
        score = lambda w: sum(a * b for a, b in zip(self.direction({**spec, 'wristR': list(w)}, pose_type), want))
        best = max(itertools.product(range(-150, 181, 30), range(-60, 61, 30), range(-150, 181, 30)), key=score)
        for step in (10, 4):
            around = [[c + d * step for d in (-2, -1, 0, 1, 2)] for c in best]
            best = max(itertools.product(*around), key=score)
        return list(best)
