use crate::source::Source;
use crate::{bundle, convert, extra, mcp, parity, routes};
use serde_json::{json, Value};
use std::sync::Arc;
use ume_formats::vfs::{add_zip, Vfs};

fn kfm() -> Source {
    let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/mugen/elecbyte.zip")).unwrap();
    let mut vfs = Vfs::new();
    add_zip(&mut vfs, &bytes, "").unwrap();
    Source { vfs, def: "chars/kfm/kfm.def".into() }
}

fn temp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mugen2umf-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn kung_fu_man_migrates_with_identical_data_and_moves() {
    let src = kfm();
    let out = temp("kfm");
    let b = convert::convert(&src, "kfm_umf", None).unwrap();
    convert::write(&b, &out, "kfm_umf").unwrap();
    let sources = ume_engine::loader::read_mugen_sources(&src.vfs, &src.def).unwrap();
    let orig = Arc::new(ume_engine::loader::assemble_mugen_char(sources.clone(), 1, true));
    let umf = Arc::new(bundle::compile(&bundle::load(&out).unwrap(), 1, true).unwrap());
    let routes = routes::routes(&sources, &orig.commands);
    assert!(routes.len() >= 10, "kfm exposes its specials and supers through Statedef -1, got {}", routes.len());
    let report = parity::parity("kfm", &orig, &umf, &umf, &routes);
    assert!(report.identical, "{}", report.markdown());
    assert!(report.moves.iter().any(|m| m.original.damage > 0.0), "at least one played move must hit the dummy");
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn every_palette_is_kept_for_palette_swaps() {
    let src = kfm();
    let out = temp("kfm-pal");
    let b = convert::convert(&src, "kfm_pal", None).unwrap();
    convert::write(&b, &out, "kfm_pal").unwrap();
    let sources = ume_engine::loader::read_mugen_sources(&src.vfs, &src.def).unwrap();
    let loaded = bundle::load(&out).unwrap();
    for pal in [2, 4, 6] {
        let orig = ume_engine::loader::assemble_mugen_char(sources.clone(), pal, true);
        let umf = bundle::compile(&loaded, pal, true).unwrap();
        let px = |d: &ume_engine::chardef::CharDef| d.palettes.get(&(1, 1)).map(|p| p.to_vec());
        assert!(px(&orig).is_some());
        assert_eq!(px(&orig), px(&umf), "palette {pal}");
    }
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn unlocked_routes_and_move_names_come_from_the_extra_file() {
    let src = kfm();
    let mut b = convert::convert(&src, "kfm_extra", None).unwrap();
    let applied = extra::apply(&mut b.json, &json!({
        "profile": {"country": "CN"},
        "mugenRoutes": [{"state": 1000, "id": "palm_again", "input": "~D, DB, B, x", "name": "Palm Again"}],
        "moves": [{"id": "QCF_x", "name": "Kung Fu Palm"}],
    })).unwrap();
    assert!(applied.contains(&"mugenRoutes".to_string()));
    let fighter = ume_native::parse_char(&b.json.to_string()).unwrap();
    assert_eq!(fighter.profile.unwrap().country, "CN");
    assert_eq!(b.json["mugen"]["names"]["QCF_x"], json!("Kung Fu Palm"));
    let cns = b.json["scripts"]["cns"][0].as_str().unwrap();
    assert!(cns.contains("value = 1000") && cns.contains("command = \"umf_palm_again\""));
}

#[test]
fn quoted_commands_skip_negated_checks() {
    assert_eq!(routes::quoted_commands(r#"command = "QCF_x" && command != "holddown" || Command="a""#), vec!["QCF_x".to_string(), "a".to_string()]);
}

#[test]
fn mcp_lists_every_tool_with_a_schema() {
    let reply = mcp::handle(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})).unwrap();
    let tools = reply["result"]["tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t["name"] == "convert" && t["inputSchema"]["required"] == json!(["src", "out", "id"])));
    let bad = mcp::handle(&json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "inspect", "arguments": {"nope": "x"}}})).unwrap();
    assert_eq!(bad["result"]["isError"], Value::Bool(true));
    assert!(mcp::handle(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).is_none());
}

#[test]
fn umf_moves_layer_on_top_of_the_exact_fighter() {
    let src = kfm();
    let mut b = convert::convert(&src, "kfm_moves", None).unwrap();
    extra::apply(&mut b.json, &json!({
        "moves": [{"id": "dragon_palm", "name": "Dragon Palm", "input": "~D, U, x", "kind": "special", "anim": "1000", "hits": [{"at": 2, "damage": 77}]}],
    })).unwrap();
    let out = temp("kfm-moves");
    convert::write(&b, &out, "kfm_moves").unwrap();
    let sources = ume_engine::loader::read_mugen_sources(&src.vfs, &src.def).unwrap();
    let orig = Arc::new(ume_engine::loader::assemble_mugen_char(sources.clone(), 1, true));
    let umf = Arc::new(bundle::compile(&bundle::load(&out).unwrap(), 1, true).unwrap());
    let block = ume_native::exact::overlay_block(&orig);
    let added = umf.movelist.iter().find(|m| m.command == "m_dragon_palm").expect("UMF move listed");
    assert!(added.state >= block, "UMF move state {} must live above every MUGEN state", added.state);
    let route = routes::Route { command: added.command.clone(), input: added.input.clone(), state: added.state, when: "stand", cost: 0 };
    let played = parity::sim::run(&umf, &route).outcome;
    assert!(played.states.contains(&added.state), "real input reaches the UMF move: {:?}", played.states);
    assert!(played.damage >= 77.0, "UMF move hits the dummy, damage {}", played.damage);
    let base = Arc::new(bundle::compile_base(&bundle::load(&out).unwrap(), 1, true).unwrap());
    let report = parity::parity("kfm", &orig, &base, &umf, &routes::routes(&sources, &orig.commands));
    assert!(report.identical, "the exact part stays identical under the overlay:\n{}", report.markdown());
    assert!(report.added.iter().any(|m| m.route.command == "m_dragon_palm" && m.reached), "{}", report.markdown());
    let with_overlay = parity::parity("kfm", &orig, &umf, &umf, &routes::routes(&sources, &orig.commands));
    assert!(with_overlay.moves.iter().all(|m| m.identical), "original moves play the same with the overlay loaded:\n{}", with_overlay.markdown());
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn unlocked_routes_run_before_the_original_commands() {
    let src = kfm();
    let mut b = convert::convert(&src, "kfm_first", None).unwrap();
    extra::apply(&mut b.json, &json!({"mugenRoutes": [{"state": 1000, "id": "palm_first", "input": "~D, DF, F, x"}]})).unwrap();
    let out = temp("kfm-first");
    convert::write(&b, &out, "kfm_first").unwrap();
    let umf = Arc::new(bundle::compile(&bundle::load(&out).unwrap(), 1, true).unwrap());
    let first = &umf.states[&-1].controllers[0];
    assert!(format!("{:?}", first.trigger_all).contains("umf_palm_first"), "the unlocked route must be checked before the original ones it shares an input with");
    let route = routes::Route { command: "umf_palm_first".into(), input: "~D, DF, F, x".into(), state: 1000, when: "stand", cost: 0 };
    assert!(parity::sim::run(&umf, &route).outcome.states.contains(&1000));
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn goku_vision_bundle_has_afterimage_trails_and_r_charge() {
    let Ok(dir) = std::env::var("UME_GOKU_VISION_BUNDLE") else { return };
    use ume_engine::common::{common_def, flat_stage};
    use ume_engine::input::k;
    use ume_engine::matchs::Match;
    let def = Arc::new(bundle::compile(&bundle::load(std::path::Path::new(&dir)).unwrap(), 1, true).unwrap());
    let duel = |def: &Arc<ume_engine::chardef::CharDef>| {
        let mut m = Match::new(common_def(), None, flat_stage());
        m.add_player(def.clone(), 0, 0, 1);
        m.add_player(def.clone(), 1, 0, 2);
        m.start_round();
        for _ in 0..800 {
            m.tick(&[0, 0]);
            if m.round_state == 2 {
                break;
            }
        }
        m.fighters[1].life_max = 100000.0;
        m.fighters[1].life = 100000.0;
        m
    };
    let mut m = duel(&def);
    m.fighters[0].power = 3000.0;
    let d = m.fighters[0].def.clone();
    m.change_state(0, 7500, d);
    let mut ghosts = 0;
    for _ in 0..60 {
        m.tick(&[0, 0]);
        ghosts = ghosts.max(m.fighters[0].after_image.as_ref().map_or(0, |a| a.trail.len()));
    }
    assert!(ghosts > 3, "Shunkan Ido leaves an afterimage trail when loaded from the bundle, got {ghosts}");
    let mut m = duel(&def);
    m.fighters[0].power = 0.0;
    let mut charging = 0;
    for _ in 0..40 {
        m.tick(&[k::S, 0]);
        charging += (m.fighters[0].state_no == 90200) as usize;
    }
    let held = m.fighters[0].power;
    assert!(charging >= 30 && held >= 300.0, "holding R charges ki: {charging} ticks in 90200, power {held}");
    for _ in 0..10 {
        m.tick(&[0, 0]);
    }
    assert!(m.fighters[0].state_no != 90200 && m.fighters[0].ctrl, "releasing R returns control, state {}", m.fighters[0].state_no);
    let mut m = duel(&def);
    m.fighters[0].power = 1000.0;
    m.fighters[0].pos.x = -60.0;
    m.fighters[1].pos.x = 60.0;
    let (mut skill, mut trail) = (0, 0);
    for t in 0..40 {
        m.tick(&[if t < 4 { k::X | k::Y } else { 0 }, 0]);
        skill += (m.fighters[0].state_no == 90300) as usize;
        trail = trail.max(m.fighters[0].after_image.as_ref().map_or(0, |a| a.trail.len()));
    }
    assert!(skill >= 15 && trail > 3, "x+y plays the After Image skill with a trail: {skill} ticks, trail {trail}");
    assert!(m.fighters[0].pos.x > m.fighters[1].pos.x, "the skill ends behind the foe: {} vs {}", m.fighters[0].pos.x, m.fighters[1].pos.x);
    assert!(m.fighters[0].power < 900.0, "the skill spends ki, power {}", m.fighters[0].power);
}
