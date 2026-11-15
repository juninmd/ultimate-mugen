use ume_engine::native::{compile_native, NativeChar};

const DOUBLE_AIR_DASHERS: [&str; 2] = ["suzume", "jin"];
const BOUNCERS: [(&str, &str); 9] = [
    ("suzume", "wallBounce"),
    ("jin", "wallBounce"),
    ("kaito", "wallBounce"),
    ("garan", "wallBounce"),
    ("garan", "groundBounce"),
    ("bruno", "wallBounce"),
    ("shion", "wallBounce"),
    ("daichi", "wallBounce"),
    ("daichi", "groundBounce"),
];

fn raw(id: &str) -> String {
    std::fs::read_to_string(format!("../../assets/chars/{id}.json")).unwrap()
}

#[test]
fn double_air_dashers_declare_two_dashes() {
    for id in DOUBLE_AIR_DASHERS {
        let nc: NativeChar = serde_json::from_str(&raw(id)).unwrap();
        assert_eq!(nc.stats.air_dashes, Some(2.0), "{id}");
        compile_native(&nc, 1, true);
    }
}

#[test]
fn bouncing_moves_carry_their_hit_flag() {
    for (id, flag) in BOUNCERS {
        assert!(raw(id).contains(&format!("\"{flag}\":true")), "{id} {flag}");
    }
}
