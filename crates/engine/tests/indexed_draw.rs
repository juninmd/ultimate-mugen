use ume_engine::loader::load_mugen_char;
use ume_formats::vfs::{add_zip, Vfs};

fn shade(px: &[u8], pal: &[u8]) -> Vec<u8> {
    px.iter().flat_map(|&c| if c == 0 { [0; 4] } else { let p = c as usize * 4; [pal.get(p).copied().unwrap_or(0), pal.get(p + 1).copied().unwrap_or(0), pal.get(p + 2).copied().unwrap_or(0), 255] }).collect()
}

#[test]
fn palette_shading_matches_baked_rgba() {
    let mut vfs = Vfs::new();
    add_zip(&mut vfs, &std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/mugen/elecbyte.zip")).unwrap(), "").unwrap();
    for def in ["chars/kfm/kfm.def", "chars/kfm720/kfm720.def"] {
        for pal in [1, 3] {
            let d = load_mugen_char(&vfs, def, pal, true).unwrap();
            let mut checked = 0;
            for (k, ix) in &d.indexed {
                let (Some(img), Some(p)) = (d.rgba_sprite(*k), ix.palette.as_ref()) else { continue };
                assert_eq!(shade(&ix.pixels, p), img.rgba, "{def} pal {pal} sprite {k:?}");
                checked += 1;
            }
            assert!(checked > 200, "{def}: {checked}");
        }
    }
}
