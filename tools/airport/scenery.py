import math
import random
from glb import Mesh
from shapes import box, hexa, lathe, place


def ground(mesh):
    for z0 in range(-100, 130, 10):
        mesh.quad("apron", (-420, 0, z0), (-420, 0, z0 + 10), (420, 0, z0 + 10), (420, 0, z0))
    mesh.quad("grass", (-900, -0.05, -100), (-900, -0.05, 130), (900, -0.05, 130), (900, -0.05, -100))
    for z in range(-420, -100, 25):
        mesh.quad("grass", (-900, -0.05, z), (-900, -0.05, z + 25), (900, -0.05, z + 25), (900, -0.05, z))
    mesh.quad("asphalt", (-900, 0.02, -140), (-900, 0.02, -100), (900, 0.02, -100), (900, 0.02, -140))
    for x in range(-420, 420, 24):
        box(mesh, "paint", (x, 0.06, -120), (12, 0.04, 0.9))
    box(mesh, "stripe", (0, 0.05, -14), (840, 0.04, 0.5))
    for x in (-40, 8, 56):
        box(mesh, "stripe", (x, 0.05, -26), (0.5, 0.04, 22))
        box(mesh, "paint", (x + 3, 0.05, -2), (0.4, 0.04, 6))
    for z in (28, 62, 96):
        box(mesh, "paint", (0, 0.05, z), (840, 0.04, 0.4))
    for x in range(-200, 201, 40):
        box(mesh, "paint", (x, 0.05, 45), (0.4, 0.04, 34))
        box(mesh, "stripe", (x + 20, 0.05, 79), (0.5, 0.04, 34))
    for x in range(-420, 420, 20):
        box(mesh, "glow_blue", (x, 0.4, -96), (0.7, 0.8, 0.7))
        box(mesh, "glow_white", (x, 0.3, -144), (0.7, 0.6, 0.7))


def terminal(mesh, cx, width, z):
    front = z + 13
    box(mesh, "concrete", (cx, 8, z), (width, 16, 26))
    box(mesh, "roof", (cx, 16.6, z + 6), (width + 10, 1.2, 46))
    box(mesh, "glass", (cx, 7, front + 0.4), (width - 6, 12, 0.6))
    for x in range(int(cx - width / 2 + 3), int(cx + width / 2 - 2), 6):
        box(mesh, "frame", (x, 7, front + 0.7), (0.5, 12.4, 0.8))
    box(mesh, "frame", (cx, 1.2, front + 0.7), (width - 6, 0.6, 0.8))
    box(mesh, "frame", (cx, 13.0, front + 0.7), (width - 6, 0.6, 0.8))
    for x in range(int(cx - width / 2 + 12), int(cx + width / 2 - 8), 24):
        box(mesh, "glow_warm", (x, 14.6, front + 1.2), (6, 1.0, 0.4))


def jet_bridge(mesh, x, z_far, z_near):
    box(mesh, "bridge", (x, 5.2, (z_far + z_near) / 2), (3.2, 3.0, abs(z_far - z_near)))
    box(mesh, "frame", (x, 3.4, z_near), (4.0, 0.5, 4.0))
    box(mesh, "dark", (x - 1.7, 1.6, z_near), (0.4, 3.2, 0.4))
    box(mesh, "dark", (x + 1.7, 1.6, z_near), (0.4, 3.2, 0.4))


def tower(mesh, x, z):
    lathe(mesh, [(0, x, z, 3.4), (22, x, z, 2.6), (48, x, z, 2.2)], 10,
          lambda k, i: "concrete", axis="y")
    lathe(mesh, [(48, x, z, 2.6), (50.5, x, z, 5.6), (55, x, z, 6.6), (58, x, z, 6.6)], 10,
          lambda k, i: "glass" if k == 2 else "frame", axis="y")
    lathe(mesh, [(58, x, z, 6.9), (59.4, x, z, 6.9)], 10, lambda k, i: "roof", axis="y")
    lathe(mesh, [(59.4, x, z, 0.3), (66, x, z, 0.3)], 6, lambda k, i: "frame", axis="y")
    box(mesh, "paint", (x, 62.5, z), (5.5, 0.4, 0.4))
    box(mesh, "paint", (x + 4.0, 61.0, z), (0.4, 0.4, 3.5))


def hills(mesh):
    rnd = random.Random(7)
    x, tops = -1400.0, []
    while x < 1400:
        tops.append((x, 20 + rnd.random() * 38))
        x += 70 + rnd.random() * 90
    for (x0, h0), (x1, h1) in zip(tops, tops[1:]):
        mesh.tri("hill", (x0, 0, -600), (x1, 0, -600), (x0, h0, -600))
        mesh.tri("hill", (x1, 0, -600), (x1, h1, -600), (x0, h0, -600))
        mesh.tri("hill", (x1, h1, -600), (x1, 0, -600), (x0, h0, -600))


def clouds(mesh):
    rnd = random.Random(11)
    for _ in range(9):
        cx, cy, cz = rnd.uniform(-500, 500), rnd.uniform(70, 150), rnd.uniform(-520, -380)
        for k in range(3):
            r = rnd.uniform(14, 26)
            x0 = cx + k * 30
            lathe(mesh, [(x0, cy, cz, 0.3 * r), (x0 + r, cy + r * 0.05, cz, r * 0.4), (x0 + 2 * r, cy, cz, 0.3 * r)],
                  7, lambda a, b: "cloud")


def sun(mesh):
    ring = [(60 + 60 * math.cos(i * math.pi / 8), 62 + 60 * math.sin(i * math.pi / 8), -780) for i in range(17)]
    for p, q in zip(ring, ring[1:]):
        mesh.tri("glow_sun", (60, 62, -780), p, q)


def props(mesh):
    for x in (-190, -130, 100, 150, 200):
        box(mesh, "frame", (x, 6, 6), (0.5, 12, 0.5))
        box(mesh, "glow_warm", (x, 12.2, 6.8), (2.2, 0.4, 1.8))
    for x, z in ((-62, -8), (-58, -8), (-4, -6), (38, -8), (74, -6)):
        box(mesh, "cone", (x, 0.5, z), (0.6, 1.0, 0.6))
    box(mesh, "truck", (30, 1.6, -22), (6.0, 3.2, 2.6))
    box(mesh, "cone", (30, 3.8, -22), (2.4, 0.8, 2.2))
    box(mesh, "cart", (-20, 1.0, -23), (2.4, 1.6, 2.0))
