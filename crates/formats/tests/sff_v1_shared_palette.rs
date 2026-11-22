use ume_formats::sff::decode_sff;

fn pcx(color: u8, palette: Option<[u8; 3]>) -> Vec<u8> {
    let mut out = vec![0u8; 128];
    out[0] = 10;
    out[1] = 5;
    out[2] = 1;
    out[3] = 8;
    out[8..10].copy_from_slice(&1u16.to_le_bytes());
    out[10..12].copy_from_slice(&1u16.to_le_bytes());
    out[65] = 1;
    out[66..68].copy_from_slice(&2u16.to_le_bytes());
    out.extend([0xC2, color]);
    out.extend([0xC2, color]);
    if let Some(rgb) = palette {
        out.push(12);
        for _ in 0..256 {
            out.extend(rgb);
        }
    }
    out
}

fn sff(palette_type: u8, sprites: &[(u16, u16, bool, Vec<u8>)]) -> Vec<u8> {
    let mut out = b"ElecbyteSpr\0".to_vec();
    out.extend([0, 1, 0, 1]);
    out.extend(1u32.to_le_bytes());
    out.extend((sprites.len() as u32).to_le_bytes());
    out.extend(512u32.to_le_bytes());
    out.extend(32u32.to_le_bytes());
    out.push(palette_type);
    out.resize(512, 0);
    for (k, (group, index, same_pal, data)) in sprites.iter().enumerate() {
        let next = if k + 1 == sprites.len() { 0 } else { out.len() + 32 + data.len() };
        out.extend((next as u32).to_le_bytes());
        out.extend((data.len() as u32).to_le_bytes());
        out.extend([0u8; 4]);
        out.extend(group.to_le_bytes());
        out.extend(index.to_le_bytes());
        out.extend(0u16.to_le_bytes());
        out.push(*same_pal as u8);
        out.extend([0u8; 13]);
        out.extend(data);
    }
    out
}

fn layout() -> Vec<(u16, u16, bool, Vec<u8>)> {
    vec![
        (0, 0, false, pcx(1, Some([200, 30, 30]))),
        (9000, 1, false, pcx(1, Some([40, 0, 40]))),
        (200, 0, true, pcx(1, None)),
    ]
}

#[test]
fn shared_palette_sprites_after_the_portrait_keep_the_character_palette() {
    let decoded = decode_sff(&sff(1, &layout())).unwrap();
    let pal = |g: i32| decoded.sprites.iter().find(|s| s.group == g).unwrap().palette_index;
    assert_eq!(pal(200), pal(0), "a shared-palette sprite uses the character palette, not the portrait palette before it");
    assert_ne!(pal(9000), pal(0), "the portrait keeps its own palette");
}

#[test]
fn individual_palette_files_still_inherit_from_the_previous_sprite() {
    let decoded = decode_sff(&sff(0, &layout())).unwrap();
    let pal = |g: i32| decoded.sprites.iter().find(|s| s.group == g).unwrap().palette_index;
    assert_eq!(pal(200), pal(9000));
}
