import math
from glb import Mesh
from shapes import box, hexa, lathe, mirror_z, tapered_wing

SEGS = 16
WINDOW = {3, 12}
CHEAT = {4, 11}
BELLY = {6, 7, 8, 9}


def fuselage(mesh, livery):
    xs = [-19.0, -16.0, -13.0] + [-11.0 + 2 * i for i in range(13)] + [15.0, 16.6, 17.8, 18.5]
    radius = {-19.0: 0.25, -16.0: 0.9, -13.0: 1.6, 15.0: 1.85, 16.6: 1.45, 17.8: 0.8, 18.5: 0.15}
    rings = [(x, 4.6 + max(0.0, -10.0 - x) * 0.16, 0.0, radius.get(x, 2.0)) for x in xs]

    def mat(k, i):
        if rings[k][0] < -10.5 or rings[k][0] > 13.5:
            return "hull" if i not in BELLY else "belly"
        if i in WINDOW:
            return "window"
        return livery if i in CHEAT else "belly" if i in BELLY else "hull"

    lathe(mesh, rings, SEGS, mat)
    cockpit = [(17.1, 5.0, 0.0, 1.2), (18.0, 4.9, 0.0, 0.8)]
    lathe(mesh, cockpit, 8, lambda k, i: "window" if i in (0, 1, 7) else "hull")


def wings_and_tail(mesh, livery):
    half = Mesh()
    tapered_wing(half, "hull", (5.0, -3.0), (-6.5, -9.5, 15.5), 3.7, 0.4, 1.5)
    tapered_wing(half, "hull", (-13.5, -18.5), (-17.0, -19.2, 6.0), 6.0, 0.25, 0.5)
    lathe(half, [(-0.5, 2.6, 6.5, 1.15), (2.2, 2.6, 6.5, 1.25), (3.6, 2.6, 6.5, 1.1)], 10,
          lambda k, i: "belly")
    lathe(half, [(3.6, 2.6, 6.5, 1.0), (3.9, 2.6, 6.5, 0.9)], 10, lambda k, i: "dark")
    mesh.merge(half, lambda v: v)
    mesh.merge(mirror_z(half), lambda v: v)
    fin = [(-11.0, 6.0, -0.2), (-16.0, 6.0, -0.2), (-19.5, 12.4, -0.2), (-17.2, 12.4, -0.2)]
    hexa(mesh, livery, [(p[0], p[1], -0.25) for p in fin] + [(p[0], p[1], 0.25) for p in fin])


def gear(mesh):
    for x, z in ((12.0, 0.0), (-2.0, 3.2), (-2.0, -3.2)):
        box(mesh, "dark", (x, 1.6, z), (0.25, 3.0, 0.25))
        for dz in (-0.4, 0.4):
            box(mesh, "tyre", (x, 0.6, z + dz), (1.2, 1.2, 0.4))


def airliner(livery):
    mesh = Mesh()
    fuselage(mesh, livery)
    wings_and_tail(mesh, livery)
    gear(mesh)
    return mesh
