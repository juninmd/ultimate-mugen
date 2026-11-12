use super::*;
use crate::atlas::Rect;

fn canvas() -> Canvas { Canvas::with_white(Rect { x: 0, y: 0, w: 1, h: 1 }, [16, 16]) }

fn alpha(v: &Vertex) -> u16 { v.mul[3] }

#[test]
fn a_skewed_quad_fades_only_its_diagonal_edges() {
    let mut c = canvas();
    c.convex(&[[20.0, 0.0], [120.0, 0.0], [100.0, 40.0], [0.0, 40.0]], Color::WHITE);
    let v = &c.batch.vertices;
    assert_eq!((v.len(), c.batch.indices.len()), (8, 6 + 24));
    for pair in v.chunks(2) {
        assert_eq!((alpha(&pair[0]), alpha(&pair[1])), (65535, 0));
        assert_eq!(pair[0].pos[1], pair[1].pos[1]);
        assert!((pair[0].pos[0] - pair[1].pos[0]).abs() > 0.9);
    }
    assert_eq!(v[0].pos[1], 0.0);
    assert_eq!(v[4].pos[1], 40.0);
}

#[test]
fn an_axis_aligned_rectangle_keeps_hard_pixel_edges() {
    let mut c = canvas();
    c.convex(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]], Color::WHITE);
    for pair in c.batch.vertices.chunks(2) { assert_eq!(pair[0].pos, pair[1].pos); }
}

#[test]
fn shapes_smaller_than_the_fringe_stay_plain() {
    let mut c = canvas();
    c.convex(&[[0.0, 0.0], [2.0, 1.0], [1.0, 2.0]], Color::WHITE);
    assert_eq!((c.batch.vertices.len(), c.batch.indices.len()), (3, 3));
}

#[test]
fn a_sharp_corner_never_spikes_past_the_miter_limit() {
    let mut c = canvas();
    c.convex(&[[0.0, 0.0], [400.0, 10.0], [0.0, 20.0]], Color::WHITE);
    for pair in c.batch.vertices.chunks(2) {
        let (dx, dy) = (pair[1].pos[0] - pair[0].pos[0], pair[1].pos[1] - pair[0].pos[1]);
        assert!((dx * dx + dy * dy).sqrt() <= 2.0 * MITER_LIMIT * FEATHER + 1e-3);
    }
}

#[test]
fn circles_rings_and_lines_carry_a_one_pixel_fringe() {
    let mut c = canvas();
    c.circle([50.0, 50.0], 20.0, Color::WHITE);
    assert_eq!((c.batch.vertices.len(), c.batch.indices.len()), (64, 30 * 3 + 32 * 6));
    let outer = c.batch.vertices.iter().filter(|v| alpha(v) == 0).count();
    assert_eq!(outer, 32);
    c.clear();
    c.circle_lines([50.0, 50.0], 20.0, 3.0, Color::WHITE);
    assert_eq!((c.batch.vertices.len(), c.batch.indices.len()), (128, 32 * 18));
    c.clear();
    c.line([0.0, 0.0], [30.0, 40.0], 0.5, Color::WHITE);
    assert_eq!((c.batch.vertices.len(), c.batch.indices.len()), (8, 18));
    assert_eq!(alpha(&c.batch.vertices[1]), 32768);
    assert!(c.batch.indices.iter().all(|&i| (i as usize) < c.batch.vertices.len()));
}

#[test]
fn a_radial_gradient_is_one_fan_plus_one_strip_per_ring() {
    let mut c = canvas();
    let stops = [(0.0, Color::WHITE), (0.5, Color([1.0, 1.0, 1.0, 0.5])), (1.0, clear(Color::WHITE))];
    c.radial_gradient([0.0, 0.0], 10.0, 8, &stops);
    assert_eq!((c.batch.vertices.len(), c.batch.indices.len()), (17, 8 * 3 + 8 * 6));
    assert_eq!(c.batch.calls.len(), 1);
    assert_eq!(c.batch.vertices[16].pos, [(7.0f32 / 8.0 * std::f32::consts::TAU).cos() * 10.0, (7.0f32 / 8.0 * std::f32::consts::TAU).sin() * 10.0]);
}
