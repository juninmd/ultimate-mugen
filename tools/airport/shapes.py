import math


def box(mesh, mat, c, s):
    x, y, z = c
    hx, hy, hz = (v / 2 for v in s)
    p = [(x + sx * hx, y + sy * hy, z + sz * hz) for sx in (-1, 1) for sy in (-1, 1) for sz in (-1, 1)]
    hexa(mesh, mat, p)


def hexa(mesh, mat, p):
    faces = [(0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)]
    for a, b, c, d in faces:
        mesh.quad(mat, p[a], p[b], p[c], p[d])


def lathe(mesh, rings, segs, mat_of, caps=("x", "x"), axis="x"):
    def at(x, cy, cz, r, i):
        t = i * 2 * math.pi / segs
        v = (x, cy + r * math.cos(t), cz + r * math.sin(t))
        return v if axis == "x" else (v[1], v[0], v[2])

    for k in range(len(rings) - 1):
        for i in range(segs):
            a, b = rings[k], rings[k + 1]
            mesh.quad(mat_of(k, i), at(*a, i), at(*a, i + 1), at(*b, i + 1), at(*b, i))
    for end, ring in ((0, rings[0]), (1, rings[-1])):
        mat = mat_of(0 if end == 0 else len(rings) - 2, 0)
        for i in range(segs):
            centre = at(ring[0], ring[1], ring[2], 0.0, 0)
            mesh.tri(mat, centre, at(*ring, i), at(*ring, i + 1))


def tapered_wing(mesh, mat, root, tip, y, thick, rise):
    (rl, rt), (tl, tt, z) = root, tip
    a = [(rl, y, 0.0), (rt, y, 0.0), (tl, y + rise, z), (tt, y + rise, z)]
    b = [(p[0], p[1] + thick, p[2]) for p in a]
    mesh.quad(mat, a[0], a[1], a[3], a[2])
    mesh.quad(mat, b[0], b[2], b[3], b[1])
    mesh.quad(mat, a[0], a[2], b[2], b[0])
    mesh.quad(mat, a[1], b[1], b[3], a[3])
    mesh.quad(mat, a[0], b[0], b[1], a[1])
    mesh.quad(mat, a[2], a[3], b[3], b[2])


def mirror_z(mesh):
    out = type(mesh)()
    out.merge(mesh, lambda v: (v[0], v[1], -v[2]))
    return out


def place(mesh, yaw=0.0, at=(0, 0, 0), scale=1.0):
    c, s = math.cos(yaw), math.sin(yaw)
    out = type(mesh)()
    out.merge(mesh, lambda v: (
        (v[0] * c + v[2] * s) * scale + at[0], v[1] * scale + at[1], (-v[0] * s + v[2] * c) * scale + at[2],
    ))
    return out
