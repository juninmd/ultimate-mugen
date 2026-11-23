#!/usr/bin/env python3
import math
import sys
from airliner import airliner
from glb import Mesh, Writer
from scenery import clouds, ground, hills, jet_bridge, props, sun, terminal, tower
from shapes import box, place

LOOP = 40.0
MATERIALS = {
    "hull": ((0.93, 0.94, 0.96), 0), "belly": ((0.62, 0.66, 0.72), 0), "window": ((0.10, 0.13, 0.20), 0),
    "dark": ((0.12, 0.12, 0.14), 0), "tyre": ((0.08, 0.08, 0.09), 0),
    "red": ((0.82, 0.16, 0.18), 0), "blue": ((0.12, 0.34, 0.78), 0), "green": ((0.10, 0.60, 0.42), 0),
    "orange": ((0.95, 0.50, 0.10), 0), "apron": ((0.50, 0.52, 0.56), 0), "asphalt": ((0.22, 0.23, 0.26), 0),
    "grass": ((0.30, 0.45, 0.25), 0), "stripe": ((0.95, 0.78, 0.15), 0), "paint": ((0.90, 0.90, 0.90), 0),
    "concrete": ((0.72, 0.70, 0.68), 0), "roof": ((0.85, 0.86, 0.88), 0), "glass": ((0.28, 0.50, 0.62), 0),
    "frame": ((0.25, 0.27, 0.30), 0), "bridge": ((0.80, 0.82, 0.85), 0), "cone": ((0.95, 0.45, 0.10), 0),
    "cart": ((0.20, 0.45, 0.75), 0), "truck": ((0.85, 0.80, 0.20), 0), "hill": ((0.35, 0.40, 0.55), 0),
    "cloud": ((1.0, 0.82, 0.78), 0), "glow_sun": ((1.0, 0.78, 0.45), 1), "glow_blue": ((0.30, 0.55, 1.0), 1),
    "glow_white": ((1.0, 1.0, 0.9), 1), "glow_warm": ((1.0, 0.82, 0.5), 1),
}


def quat_y(a):
    return (0.0, math.sin(a / 2), 0.0, math.cos(a / 2))


def quat_z(a):
    return (0.0, 0.0, math.sin(a / 2), math.cos(a / 2))


def quat_mul(a, b):
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return (aw * bx + ax * bw + ay * bz - az * by, aw * by - ax * bz + ay * bw + az * bx,
            aw * bz + ax * by - ay * bx + az * bw, aw * bw - ax * bx - ay * by - az * bz)


def landing_keys():
    return [(0.0, (-260, 72, -120), 0.04), (9.0, (50, 7, -120), 0.09), (9.7, (96, 0.0, -120), 0.0),
            (15.0, (210, 0.0, -120), 0.0), (17.0, (320, 0.0, -120), 0.0), (17.1, (320, -300, -120), 0.0),
            (39.9, (320, -300, -120), 0.0), (40.0, (-260, 72, -120), 0.04)]


def build(out):
    static = Mesh()
    for fn in (ground, hills, clouds, sun, props):
        fn(static)
    terminal(static, -50, 130, -72)
    tower(static, 52, -74)
    for x in (-40, 12):
        jet_bridge(static, x + 9, -57, -39)
    static.merge(place(airliner("red"), yaw=0.14, at=(-40, 0, -36)), lambda v: v)
    static.merge(place(airliner("blue"), yaw=-0.1, at=(12, 0, -36)), lambda v: v)
    w = Writer(MATERIALS)
    w.add_node("airport", static)
    landing = w.add_node("landing", airliner("green"))
    taxi = w.add_node("taxi", airliner("orange"))
    tug = Mesh()
    box(tug, "truck", (0, 0.9, 0), (3.0, 1.8, 1.8))
    for i in range(1, 4):
        box(tug, "cart", (-i * 3.6, 0.8, 0), (2.8, 1.4, 1.8))
    tug_node = w.add_node("tug", tug)
    keys = landing_keys()
    times = [k[0] for k in keys]
    flip = quat_y(math.pi)
    w.animate("ambient", [
        (landing, "translation", times, [k[1] for k in keys]),
        (landing, "rotation", times, [quat_z(k[2]) for k in keys]),
        (taxi, "translation", [0.0, LOOP], [(190, 0, -16), (-190, 0, -16)]),
        (taxi, "rotation", [0.0, LOOP], [flip, flip]),
        (tug_node, "translation", [0.0, LOOP], [(-110, 0, -6), (110, 0, -6)]),
    ])
    w.save(out, MATERIALS)


if __name__ == "__main__":
    build(sys.argv[1] if len(sys.argv) > 1 else "airport.glb")
