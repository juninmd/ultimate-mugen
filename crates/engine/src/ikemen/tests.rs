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

#[test]
fn text_lifecycle_and_format() {
    let (mut m, i) = setup();
    crate::controllers::run(&mut m, i, &sctrl("text", &[("text", "\"Hits: %d\""), ("params", "3+4"), ("pos", "10,20"), ("removetime", "2"), ("id", "5")]));
    assert_eq!(m.texts.len(), 1);
    assert_eq!(m.texts[0].text, "Hits: 7");
    assert_eq!(ev(&m, i, "numtext"), 1.0);
    crate::ikemen::tick(&mut m);
    assert_eq!(m.texts.len(), 1);
    crate::ikemen::tick(&mut m);
    assert!(m.texts.is_empty());
    crate::controllers::run(&mut m, i, &sctrl("text", &[("text", "\"a\""), ("removetime", "-1"), ("id", "1")]));
    crate::controllers::run(&mut m, i, &sctrl("text", &[("text", "\"b\""), ("removetime", "-1"), ("id", "2")]));
    crate::controllers::run(&mut m, i, &sctrl("removetext", &[("id", "1")]));
    assert_eq!(m.texts.len(), 1);
    assert_eq!(m.texts[0].text, "b");
    crate::controllers::run(&mut m, i, &sctrl("removetext", &[]));
    assert!(m.texts.is_empty());
}

#[test]
fn tag_out_hides_and_protects() {
    let (mut m, i) = setup();
    let mut hd = crate::fighter::HitDefData::default();
    hd.hitflag = "MAFD".into();
    hd.affect_team = 'E';
    m.round_state = 2;
    assert!(m.can_be_hit(i, &hd, 1));
    crate::controllers::run(&mut m, i, &sctrl("tagout", &[]));
    assert_eq!(ev(&m, i, "standby"), 1.0);
    assert!(!m.can_be_hit(i, &hd, 1));
    crate::controllers::run(&mut m, i, &sctrl("tagin", &[]));
    assert_eq!(ev(&m, i, "standby"), 0.0);
}

#[test]
fn format_text_specifiers() {
    assert_eq!(crate::ikemen::format_text("%d%% %f", &[50.0, 1.5]), "50% 1.500000");
    assert_eq!(crate::ikemen::format_text("x\\ny", &[]), "x\ny");
}

fn team_setup() -> (Match, usize, usize, usize) {
    let (mut m, a) = setup();
    let cd = m.fighters[a].def.clone();
    let b = m.add_player(cd, 0, 0, 3);
    let h = m.fighters.len();
    let mut helper = crate::fighter::Fighter::new(900, m.fighters[a].def.clone(), 0, 1);
    helper.is_helper = true;
    helper.parent = Some(m.fighters[a].id);
    helper.root = m.fighters[a].id;
    m.fighters.push(helper);
    (m, a, b, h)
}

#[test]
fn team_map_is_shared_between_teammates_only() {
    let (mut m, a, b, _) = team_setup();
    crate::controllers::run(&mut m, a, &sctrl("teammapset", &[("map", "\"Meter\""), ("value", "4")]));
    crate::controllers::run(&mut m, b, &sctrl("teammapadd", &[("map", "meter"), ("value", "3")]));
    assert_eq!(ev(&m, a, "teammap(meter)"), 7.0);
    assert_eq!(ev(&m, b, "teammap(meter)"), 7.0);
    assert_eq!(ev(&m, a, "map(meter)"), 0.0);
    assert_eq!(ev(&m, 1, "teammap(meter)"), 0.0);
}

#[test]
fn root_and_parent_maps_write_to_other_fighters() {
    let (mut m, a, b, h) = team_setup();
    crate::controllers::run(&mut m, h, &sctrl("rootmapset", &[("map", "k"), ("value", "9")]));
    crate::controllers::run(&mut m, h, &sctrl("parentmapadd", &[("map", "p"), ("value", "2")]));
    assert_eq!(ev(&m, a, "map(k)"), 9.0);
    assert_eq!(ev(&m, a, "map(p)"), 2.0);
    assert_eq!(ev(&m, h, "map(k)"), 0.0);
    assert_eq!(ev(&m, b, "map(k)"), 0.0);
    assert_eq!(ev(&m, h, "root, map(k)"), 9.0);
    crate::controllers::run(&mut m, a, &sctrl("parentmapset", &[("map", "orphan"), ("value", "1")]));
    assert_eq!(ev(&m, a, "map(orphan)"), 0.0);
}

#[test]
fn map_reset_clears_one_key_or_whole_scope() {
    let (mut m, a, b, _) = team_setup();
    crate::controllers::run(&mut m, a, &sctrl("mapset", &[("map", "x"), ("value", "1")]));
    crate::controllers::run(&mut m, a, &sctrl("mapset", &[("map", "y"), ("value", "2")]));
    crate::controllers::run(&mut m, b, &sctrl("mapset", &[("map", "x"), ("value", "5")]));
    crate::controllers::run(&mut m, a, &sctrl("mapreset", &[("map", "x")]));
    assert_eq!(ev(&m, a, "map(x)"), 0.0);
    assert_eq!(ev(&m, a, "map(y)"), 2.0);
    assert_eq!(ev(&m, b, "map(x)"), 5.0);
    crate::controllers::run(&mut m, a, &sctrl("mapreset", &[]));
    assert_eq!(ev(&m, a, "map(y)"), 0.0);
    crate::controllers::run(&mut m, a, &sctrl("teammapset", &[("map", "t1"), ("value", "1")]));
    crate::controllers::run(&mut m, a, &sctrl("teammapset", &[("map", "t2"), ("value", "2")]));
    crate::controllers::run(&mut m, b, &sctrl("teammapreset", &[("map", "t1")]));
    assert_eq!(ev(&m, a, "teammap(t1)"), 0.0);
    assert_eq!(ev(&m, a, "teammap(t2)"), 2.0);
    crate::controllers::run(&mut m, b, &sctrl("teammapreset", &[]));
    assert_eq!(ev(&m, a, "teammap(t2)"), 0.0);
}

#[test]
fn map_store_bounds_keys() {
    let mut s = crate::ikemen::MapStore::default();
    for k in 0..crate::ikemen::MAP_MAX_KEYS + 20 { s.set(&format!("k{k}"), 1.0, false); }
    assert_eq!(s.len(), crate::ikemen::MAP_MAX_KEYS);
    s.set(&"a".repeat(crate::ikemen::MAP_MAX_KEY_LEN + 1), 1.0, false);
    assert_eq!(s.len(), crate::ikemen::MAP_MAX_KEYS);
    s.set("k0", 5.0, true);
    assert_eq!(s.get("k0"), 6.0);
}

#[test]
fn root_var_set_and_add_write_to_root() {
    let (mut m, a, _, h) = team_setup();
    crate::controllers::run(&mut m, h, &sctrl("rootvarset", &[("var(3)", "10"), ("fvar(2)", "1.5")]));
    crate::controllers::run(&mut m, h, &sctrl("rootvaradd", &[("v", "3"), ("value", "5")]));
    assert_eq!(m.fighters[a].vars[3], 15);
    assert_eq!(m.fighters[a].fvars[2], 1.5);
    assert_eq!(m.fighters[h].vars[3], 0);
}
