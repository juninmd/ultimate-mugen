mod support;
use support::*;
use ume_engine::expr::{parse, Expr};
use ume_engine::matchs::Match;

fn ev(m: &Match, i: usize, src: &str) -> f64 { m.ev(i, &Expr::new(parse(src).unwrap())) }

fn field() -> Match { duel(mugen_char("a", ""), mugen_char("b", "")) }

#[test]
fn front_edge_is_the_right_edge_when_facing_right() {
    let mut m = field();
    m.fighters[0].facing = 1.0;
    assert_eq!(ev(&m, 0, "FrontEdge = RightEdge"), 1.0);
    assert_eq!(ev(&m, 0, "BackEdge = LeftEdge"), 1.0);
    assert!(ev(&m, 0, "FrontEdge") > ev(&m, 0, "BackEdge"));
}

#[test]
fn front_edge_is_the_left_edge_when_facing_left() {
    let mut m = field();
    m.fighters[0].facing = -1.0;
    assert_eq!(ev(&m, 0, "FrontEdge = LeftEdge"), 1.0);
    assert_eq!(ev(&m, 0, "BackEdge = RightEdge"), 1.0);
    assert!(ev(&m, 0, "FrontEdge") < ev(&m, 0, "BackEdge"));
}

#[test]
fn left_and_right_edges_ignore_facing() {
    let mut m = field();
    m.fighters[0].facing = 1.0;
    let (left, right) = (ev(&m, 0, "LeftEdge"), ev(&m, 0, "RightEdge"));
    m.fighters[0].facing = -1.0;
    assert_eq!((ev(&m, 0, "LeftEdge"), ev(&m, 0, "RightEdge")), (left, right));
    assert!(left < right);
}

#[test]
fn front_edge_agrees_with_front_edge_dist_for_both_facings() {
    let mut m = field();
    for facing in [1.0f32, -1.0] {
        m.fighters[0].facing = facing;
        m.fighters[0].pos.x = m.camera.x + 37.0;
        let pos = m.fighters[0].pos.x as f64;
        let by_edge = (ev(&m, 0, "FrontEdge") - pos) * facing as f64;
        assert!((ev(&m, 0, "FrontEdgeDist") - by_edge).abs() < 1e-4, "facing {facing}");
        let back = (ev(&m, 0, "BackEdge") - pos) * -(facing as f64);
        assert!((ev(&m, 0, "BackEdgeDist") - back).abs() < 1e-4, "facing {facing}");
    }
}
