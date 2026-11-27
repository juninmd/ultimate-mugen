use ume_native::{anim_numbers, compile_native, parse_char};

#[test]
fn bankai_getsuga_keeps_a_dark_core_and_red_edge_after_sprite_compilation() {
    use ume_engine::chardef::Blend;

    parse_char(include_str!("../../../assets/chars/kaito.json")).expect("kaito parses");
    let ichigo = parse_char(include_str!("../../../assets/chars3d/ichigo3d.json")).unwrap();
    let anims = anim_numbers(&ichigo);
    let def = compile_native(&ichigo, 1, true);
    for effect in ["getsugaFx", "getsugaBigFx"] {
        let frame = &def.anims[&anims[effect]].frames[0];
        assert_eq!(frame.blend, Blend::None);
        let pixels = def.sprites[&(frame.group, frame.index)].image.as_ref().unwrap().rgba();
        assert!(pixels.chunks_exact(4).any(|p| p[0] < 40 && p[3] > 240));
        assert!(pixels.chunks_exact(4).any(|p| p[0] > 180 && p[1] < 90 && p[3] > 200));
    }
}
