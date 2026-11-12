use ume_engine::native::{compile_native, NativeChar};

#[test]
fn every_special_and_super_points_at_a_real_state() {
    for raw in [
        include_str!("../../../assets/chars/kaito.json"),
        include_str!("../../../assets/chars/oboro.json"),
        include_str!("../../../assets/chars/kaname.json"),
    ] {
        let nc: NativeChar = serde_json::from_str(raw).expect("char json");
        let def = compile_native(&nc, 1, false);
        let shouting: Vec<_> = def.movelist.iter().filter(|m| m.command.starts_with("m_")).collect();
        assert!(!shouting.is_empty());
        for m in shouting {
            assert!(def.states.contains_key(&m.state), "{} -> state {}", m.command, m.state);
        }
    }
}
