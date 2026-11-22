use ume_native::{compile_native, parse_char};

fn cycle(def: &ume_engine::chardef::CharDef, no: i32) -> i32 {
    def.anims.get(&no).map_or(0, |a| a.frames.iter().map(|f| f.time.max(0)).sum())
}

// MUGEN median over 75 real chars: idle (action 0) 48 ticks, walk (20) 40, walk back (21) 40.
#[test]
fn locomotion_loops_keep_mugen_cadence_regardless_of_move_speed() {
    let mut checked = 0;
    for e in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/chars")).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_none_or(|x| x != "json") || p.file_name().is_some_and(|n| n == "index.json" || n.to_string_lossy().starts_with("npc_")) { continue; }
        let Ok(nc) = parse_char(&std::fs::read_to_string(&p).unwrap()) else { continue };
        if nc.speed.is_none_or(|s| s <= 1.0 || s > 1.25) { continue; }
        let def = compile_native(&nc, 1, false);
        let (idle, walk, back) = (cycle(&def, 0), cycle(&def, 20), cycle(&def, 21));
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        assert!(idle >= 48, "{name} idle cycle {idle} < 48 ticks");
        assert!(walk >= 40, "{name} walk cycle {walk} < 40 ticks");
        assert!(back >= 40, "{name} walkBack cycle {back} < 40 ticks");
        checked += 1;
    }
    assert!(checked > 5);
}
