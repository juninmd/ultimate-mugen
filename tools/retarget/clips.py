from gltf_math import load_glb, read_accessor, qslerp, Skeleton


class ClipSource:
    def __init__(self, path):
        self.doc, self.binary = load_glb(path)
        self.skeleton = Skeleton(self.doc)
        self.by_name = {n.get('name'): i for i, n in enumerate(self.doc['nodes'])}
        self.clips = {a['name']: a for a in self.doc.get('animations', [])}
        self.cache = {}

    def tracks(self, clip):
        if clip in self.cache:
            return self.cache[clip]
        anim = self.clips[clip]
        out = {}
        for ch in anim['channels']:
            s = anim['samplers'][ch['sampler']]
            times = [t[0] for t in read_accessor(self.doc, self.binary, s['input'])]
            values = read_accessor(self.doc, self.binary, s['output'])
            out[(ch['target']['node'], ch['target']['path'])] = (times, values, s.get('interpolation', 'LINEAR'))
        duration = max((v[0][-1] for v in out.values() if v[0]), default=0.0)
        self.cache[clip] = (out, duration)
        return self.cache[clip]

    def duration(self, clip):
        return self.tracks(clip)[1]

    def local_pose(self, clip, t):
        tracks, _ = self.tracks(clip)
        pose = list(self.skeleton.rest)
        for (node, path), (times, values, interp) in tracks.items():
            value = sample(times, values, t, path == 'rotation', interp == 'STEP')
            tr, rot, sc = pose[node]
            if path == 'translation':
                tr = value
            elif path == 'rotation':
                rot = value
            elif path == 'scale':
                sc = value
            pose[node] = (tr, rot, sc)
        return pose


def sample(times, values, t, rotation, step):
    if t <= times[0]:
        return tuple(values[0])
    if t >= times[-1]:
        return tuple(values[-1])
    hi = next(i for i, x in enumerate(times) if x > t)
    lo = hi - 1
    if step:
        return tuple(values[lo])
    k = (t - times[lo]) / max(times[hi] - times[lo], 1e-9)
    if rotation:
        return qslerp(tuple(values[lo]), tuple(values[hi]), k)
    return tuple(a + (b - a) * k for a, b in zip(values[lo], values[hi]))
