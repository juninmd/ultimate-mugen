use ume_engine::native::compile_native;
use ume_engine::native::NativeChar;

#[test]
fn test_all_16_pixel_art_characters() {
    std::env::remove_var("UME_PIXEL_ART");
    std::env::remove_var("UME_FORCE_RIG");

    let all_16_chars = [
        "kaito", "shion", "akio", "suzume", "bruno", "sylene", "volta", "lian",
        "garan", "marlow", "kaname", "jin", "daichi", "valdric", "haruki", "oboro"
    ];

    for cid in all_16_chars {
        let json = std::fs::read_to_string(format!("../../assets/chars/{cid}.json")).expect("read json");
        let nc: NativeChar = serde_json::from_str(&json).expect("parse json");
        let def = compile_native(&nc, 1, true);
        assert!(!def.sprites.is_empty(), "compiled sprites should not be empty for {cid}");

        // Check that at least one sprite has an image loaded
        let has_image = def.sprites.values().any(|s| s.image.is_some());
        assert!(has_image, "should have loaded image data for {cid}");
    }
}
