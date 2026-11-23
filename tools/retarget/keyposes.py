import json
import math
import sys
from pathlib import Path

from gltf_math import compact, from_to, load_glb, qmul, qnorm, qrot, save_glb
from blade_solve import BladeSolver
from retarget import FPS, Retarget, Writer

IDENTITY = (0.0, 0.0, 0.0, 1.0)
REST = {
    'hips': (0.0, 1.0, 0.0), 'spine1': (0.0, 1.08, 0.0), 'spine2': (0.0, 1.2, 0.0), 'spine3': (0.0, 1.32, 0.0),
    'neck': (0.0, 1.5, 0.0), 'head': (0.0, 1.6, 0.0),
    'shoulderL': (0.05, 1.45, 0.0), 'upperArmL': (0.18, 1.45, 0.0), 'forearmL': (0.48, 1.45, 0.0),
    'handL': (0.75, 1.45, 0.0), 'fingerL': (0.85, 1.45, 0.0),
    'shoulderR': (-0.05, 1.45, 0.0), 'upperArmR': (-0.18, 1.45, 0.0), 'forearmR': (-0.48, 1.45, 0.0),
    'handR': (-0.75, 1.45, 0.0), 'fingerR': (-0.85, 1.45, 0.0),
    'thighL': (0.1, 1.0, 0.0), 'shinL': (0.1, 0.52, 0.0), 'footL': (0.1, 0.05, 0.0), 'toeL': (0.1, 0.0, 0.14),
    'thighR': (-0.1, 1.0, 0.0), 'shinR': (-0.1, 0.52, 0.0), 'footR': (-0.1, 0.05, 0.0), 'toeR': (-0.1, 0.0, 0.14),
}
JOINTS = list(REST)
ARM = (0.30, 0.27, 0.10)
LEG = (0.48, 0.47, 0.15)
DEFAULT = {
    'hips': [0, 0], 'torso': [0, 0, 0], 'chest': [0, 0, 0], 'head': [0, 0, 0],
    'armL': [[80, -70], [60, -60]], 'armR': [[-80, -70], [-60, -60]], 'wristL': [0, 0, 0], 'wristR': [0, 0, 0],
    'legL': [[10, -90], [10, -90], [0, 0]], 'legR': [[-10, -90], [-10, -90], [0, 0]],
}


def add(a, b):
    return (a[0] + b[0], a[1] + b[1], a[2] + b[2])


def sub(a, b):
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def scale(a, k):
    return (a[0] * k, a[1] * k, a[2] * k)


def direction(azim, elev):
    az, el = math.radians(azim), math.radians(elev)
    return (math.sin(az) * math.cos(el), math.sin(el), math.cos(az) * math.cos(el))


def euler(yaw, pitch, roll):
    def axis(v, deg):
        h = math.radians(deg) / 2
        return (v[0] * math.sin(h), v[1] * math.sin(h), v[2] * math.sin(h), math.cos(h))
    return qnorm(qmul(qmul(axis((0, 1, 0), yaw), axis((1, 0, 0), pitch)), axis((0, 0, 1), roll)))


def fraction(e, k):
    return [v * k for v in e]


class Pose:
    def __init__(self, spec):
        self.spec = {**DEFAULT, **spec}

    def world(self, lift):
        s = self.spec
        out = {}
        hips_rot = euler(*s['torso'])
        hips = (0.0, 1.0 + s['hips'][1] + lift, s['hips'][0])
        out['hips'] = (hips, hips_rot)
        rot, pos = hips_rot, hips
        for name in ('spine1', 'spine2', 'spine3'):
            pos = add(pos, qrot(rot, sub(REST[name], REST['hips' if name == 'spine1' else JOINTS[JOINTS.index(name) - 1]])))
            rot = qnorm(qmul(rot, euler(*fraction(s['chest'], 1 / 3))))
            out[name] = (pos, rot)
        chest_pos, chest_rot = pos, rot
        for name, parent in (('neck', 'spine3'), ('head', 'neck')):
            pos = add(pos, qrot(rot, sub(REST[name], REST[parent])))
            rot = qnorm(qmul(rot, euler(*fraction(s['head'], 1 / 2))))
            out[name] = (pos, rot)
        for side in 'LR':
            shoulder = add(chest_pos, qrot(chest_rot, sub(REST['shoulder' + side], REST['spine3'])))
            upper = add(chest_pos, qrot(chest_rot, sub(REST['upperArm' + side], REST['spine3'])))
            out['shoulder' + side] = (shoulder, chest_rot)
            (ua, fa), wrist = s['arm' + side], s['wrist' + side]
            d_upper, d_fore = direction(*ua), direction(*fa)
            elbow = add(upper, scale(d_upper, ARM[0]))
            hand = add(elbow, scale(d_fore, ARM[1]))
            rest_dir = (1.0, 0.0, 0.0) if side == 'L' else (-1.0, 0.0, 0.0)
            upper_rot = from_to(rest_dir, d_upper)
            fore_rot = from_to(rest_dir, d_fore)
            hand_rot = qnorm(qmul(fore_rot, euler(*wrist)))
            out['upperArm' + side] = (upper, upper_rot)
            out['forearm' + side] = (elbow, fore_rot)
            out['hand' + side] = (hand, hand_rot)
            out['finger' + side] = (add(hand, scale(qrot(hand_rot, rest_dir), ARM[2])), hand_rot)
            thigh = add(hips, qrot(hips_rot, sub(REST['thigh' + side], REST['hips'])))
            (th, sh, ft) = s['leg' + side]
            knee = add(thigh, scale(direction(*th), LEG[0]))
            ankle = add(knee, scale(direction(*sh), LEG[1]))
            toe = add(ankle, scale(direction(ft[0], ft[1]), LEG[2]))
            for name, p in (('thigh', thigh), ('shin', knee), ('foot', ankle), ('toe', toe)):
                out[name + side] = (p, IDENTITY)
        return out


class KeySource:
    def __init__(self, names):
        self.by_name = {names[j]: i for i, j in enumerate(JOINTS)}
        self.rest = [(REST[j], IDENTITY, (1, 1, 1)) for j in JOINTS]

    def frame(self, pose):
        lowest = min(p[0][1] for k, p in pose.world(0.0).items() if k.startswith(('foot', 'toe')))
        world = pose.world(-lowest if pose.spec.get('ground', True) else 0.0)
        return [(world[j][0], world[j][1], (1, 1, 1)) for j in JOINTS]


def lerp_spec(a, b, k):
    if isinstance(a, list):
        return [lerp_spec(x, y, k) for x, y in zip(a, b)]
    if isinstance(a, bool):
        return a
    return a + (b - a) * k


def ease(k, kind):
    if kind == 'in':
        return k * k
    if kind == 'out':
        return 1 - (1 - k) * (1 - k)
    if kind == 'snap':
        return 1 - (1 - k) ** 4
    return k * k * (3 - 2 * k)


def sample(keys, poses, t):
    if t <= keys[0]['t']:
        return {**DEFAULT, **poses[keys[0]['pose']]}
    for a, b in zip(keys, keys[1:]):
        if t <= b['t']:
            k = ease((t - a['t']) / max(b['t'] - a['t'], 1e-6), b.get('ease', 'smooth'))
            pa, pb = {**DEFAULT, **poses[a['pose']]}, {**DEFAULT, **poses[b['pose']]}
            return {key: lerp_spec(pa[key], pb[key], k) for key in DEFAULT}
    return {**DEFAULT, **poses[keys[-1]['pose']]}


def resolve(poses, key):
    pose = poses[key]
    return {**resolve(poses, pose['base']), **pose} if 'base' in pose else pose


def bake(doc, binary, rig_name, rig, names, data):
    old = doc.get('animations', [])
    target = Retarget(doc, rig, names)
    src = KeySource(names)
    srest = src.rest
    solver = BladeSolver(doc, binary, target, src, 'Sword_066')
    poses = {}
    for key in data['poses']:
        pose = resolve(data['poses'], key)
        spec = {**DEFAULT, **pose}
        if 'bladeR' in pose:
            spec['wristR'] = solver.solve(spec, direction(*pose['bladeR']), Pose)
        poses[key] = spec
    writer = Writer(doc, binary)
    made = []
    for name, clip in data['clips'].items():
        keys = clip.get('keys_' + rig_name, clip['keys'])
        duration = keys[-1]['t']
        count = max(2, int(math.ceil(duration * FPS)) + 1)
        times, rotations, translations = [], {i: [] for i in target.driven}, []
        for f in range(count):
            t = min(f / FPS, duration)
            spec = sample(keys, poses, t)
            shake = clip.get('shake', 0.0)
            if shake:
                spec['chest'] = [spec['chest'][0], spec['chest'][1] + shake * math.sin(t * 90.0), spec['chest'][2]]
            local, translation = target.frame(src, src.frame(Pose(spec)), srest)
            times.append(t)
            for i in target.driven:
                rotations[i].append(local[i])
            translations.append(translation)
        writer.clip(name, times, rotations, target.root, translations)
        made.append(name)
    fresh = doc['animations']
    doc['animations'] = [a for a in old if a['name'] not in made] + fresh
    return writer.finish()


def main():
    dst_in, dst_out, rig_name = sys.argv[1:4]
    here = Path(__file__).parent
    rigs = json.loads((here / 'rigs.json').read_text())
    data = json.loads((here / 'keyposes.json').read_text())
    doc, binary = load_glb(dst_in)
    doc, binary = compact(*bake(doc, binary, rig_name, rigs[rig_name], rigs['sources']['ual1'], data))
    save_glb(dst_out, doc, binary)
    print(f'{dst_out}: {len(doc["animations"])} clips')


if __name__ == '__main__':
    main()
