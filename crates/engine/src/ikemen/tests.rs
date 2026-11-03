use crate::chardef::{safe_expr, CharDef, Controller, Param};
use crate::matchs::{Match, StageInfo};
use std::sync::Arc;

fn sctrl(kind: &str, params: &[(&str, &str)]) -> Controller {
    Controller { kind: kind.into(), label: kind.into(), trigger_all: vec![], triggers: vec![], params: params.iter().map(|(k, v)| (k.to_string(), Param::new(v))).collect(), persistent: 1, ignore_hitpause: false, uid: 1 }
}

fn setup() -> (Match, usize) {
    let mut cd = CharDef::empty("p");
    cd.constants.insert("data.life".into(), 1000.0);
    cd.constants.insert("data.dizzypoints".into(), 500.0);
    let cd = Arc::new(cd);
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let i = m.add_player(cd.clone(), 0, 0, 1);
    m.add_player(cd, 1, 0, 2);
    (m, i)
}

fn ev(m: &Match, i: usize, s: &str) -> f64 { m.ev(i, &safe_expr(s)) }

#[test]
fn map_set_add_and_trigger() {
    let (mut m, i) = setup();
    crate::controllers::run(&mut m, i, &sctrl("mapset", &[("map", "\"Combo\""), ("value", "3")]));
    crate::controllers::run(&mut m, i, &sctrl("mapadd", &[("map", "combo"), ("value", "2")]));
    assert_eq!(ev(&m, i, "map(combo)"), 5.0);
    assert_eq!(ev(&m, i, "Map(\"COMBO\") = 5"), 1.0);
    assert_eq!(ev(&m, i, "map(other)"), 0.0);
}

#[test]
fn points_and_flags() {
    let (mut m, i) = setup();
    assert_eq!(ev(&m, i, "dizzypointsmax"), 500.0);
    assert_eq!(ev(&m, i, "guardpointsmax"), 1000.0);
    assert_eq!(ev(&m, i, "redlife"), 1000.0);
    crate::controllers::run(&mut m, i, &sctrl("dizzypointsadd", &[("value", "-600")]));
    assert_eq!(ev(&m, i, "dizzypoints"), 0.0);
    crate::controllers::run(&mut m, i, &sctrl("guardpointsset", &[("value", "250")]));
    assert_eq!(ev(&m, i, "guardpoints"), 250.0);
    crate::controllers::run(&mut m, i, &sctrl("dizzyset", &[("value", "1")]));
    crate::controllers::run(&mut m, i, &sctrl("guardbreakset", &[("value", "1")]));
    assert_eq!(ev(&m, i, "dizzy && guardbreak"), 1.0);
    m.fighters[i].life = 600.0;
    crate::controllers::run(&mut m, i, &sctrl("redlifeadd", &[("value", "-900"), ("absolute", "1")]));
    assert_eq!(ev(&m, i, "redlife"), 600.0);
}
