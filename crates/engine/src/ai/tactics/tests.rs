use super::{affordable, is_anti_air_motion};
use crate::commands::CommandDef;
use crate::common::{common_def, flat_stage};
use crate::fighter::Fighter;
use crate::matchs::Match;
use crate::chardef::{CharDef, MoveInfo};
use crate::commands::parse_command;
use std::sync::Arc;

fn listed(name: &str, cost: f32) -> MoveInfo {
    MoveInfo { name: name.into(), input: String::new(), command: name.into(), cost, drain: false, forms: 0, state: 0 }
}

fn arena() -> Match {
    let mut cd = CharDef::empty("tactician");
    cd.commands.push(parse_command("m_typhoon", "~D, DF, F, x", 1, 1));
    cd.commands.push(parse_command("m_ex_gale", "~D, DF, F, y", 1, 1));
    cd.commands.push(parse_command("m_rise", "~F, D, DF, x", 1, 1));
    cd.commands.push(parse_command("m_gale", "~D, DF, F, z", 1, 1));
    cd.movelist.push(listed("m_typhoon", 1000.0));
    cd.movelist.push(listed("m_ex_gale", 300.0));
    let def = Arc::new(cd);
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 2);
    m.round_state = 2;
    m
}

fn command<'a>(f: &'a Fighter, name: &str) -> &'a CommandDef {
    f.def.commands.iter().find(|c| c.name == name).unwrap()
}

#[test]
fn supers_need_a_bar_and_a_reason() {
    let mut m = arena();
    let sup = command(&m.fighters[0], "m_typhoon").clone();
    m.fighters[0].power = 500.0;
    assert!(!affordable(&m.fighters[0], &m.fighters[1], &sup));
    m.fighters[0].power = 3000.0;
    assert!(!affordable(&m.fighters[0], &m.fighters[1], &sup));
    m.fighters[1].move_type = 'H';
    assert!(affordable(&m.fighters[0], &m.fighters[1], &sup));
    m.fighters[1].move_type = 'I';
    m.fighters[1].life = m.fighters[1].life_max * 0.3;
    assert!(affordable(&m.fighters[0], &m.fighters[1], &sup));
}

#[test]
fn desperate_fighters_spend_supers() {
    let mut m = arena();
    let sup = command(&m.fighters[0], "m_typhoon").clone();
    m.fighters[0].power = 1000.0;
    m.fighters[0].life = m.fighters[0].life_max * 0.4;
    assert!(affordable(&m.fighters[0], &m.fighters[1], &sup));
}

#[test]
fn ex_moves_keep_one_bar_in_reserve() {
    let mut m = arena();
    let ex = command(&m.fighters[0], "m_ex_gale").clone();
    m.fighters[0].power = 1000.0;
    assert!(!affordable(&m.fighters[0], &m.fighters[1], &ex));
    m.fighters[0].power = 1300.0;
    assert!(affordable(&m.fighters[0], &m.fighters[1], &ex));
}

#[test]
fn dragon_punch_motions_are_anti_airs() {
    let m = arena();
    let f = &m.fighters[0];
    assert!(is_anti_air_motion(command(f, "m_rise")));
    assert!(!is_anti_air_motion(command(f, "m_gale")));
}
