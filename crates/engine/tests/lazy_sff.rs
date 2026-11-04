//! A character loaded with per-sprite decoding must be identical to a fully decoded one.

use ume_engine::loader::{load_mugen_char_with, LAZY_SFF_BYTES};
use ume_formats::vfs::{add_zip, find_char_defs, Vfs};

fn vfs() -> Vfs {
    let mut vfs = Vfs::new();
    add_zip(&mut vfs, &std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/mugen/elecbyte.zip")).unwrap(), "").unwrap();
    vfs
}

#[test]
fn lazy_decoding_matches_a_full_decode() {
    let vfs = vfs();
    let defs = find_char_defs(&vfs);
    assert!(!defs.is_empty());
    for def in defs {
        for pal in [1, 2] {
            let eager = load_mugen_char_with(&vfs, &def, pal, true, usize::MAX).unwrap();
            assert!(!eager.is_lazy());
            let lazy = load_mugen_char_with(&vfs, &def, pal, true, 0).unwrap();
            assert!(lazy.is_lazy(), "{def} pal {pal}: lazy path not used");
            assert_eq!(eager.sprites.len(), lazy.sprites.len(), "{def} pal {pal}: sprite count");
            for (key, s) in &eager.sprites {
                let l = lazy.sprites.get(key).unwrap_or_else(|| panic!("{def} pal {pal}: missing {key:?}"));
                assert_eq!(
                    (s.width, s.height, s.axis_x, s.axis_y),
                    (l.width, l.height, l.axis_x, l.axis_y),
                    "{def} pal {pal} {key:?}: metadata"
                );
            }
            let mut keys: Vec<(i32, i32)> = eager.anims.values().flat_map(|a| a.frames.iter().map(|f| (f.group, f.index))).collect();
            keys.sort();
            keys.dedup();
            assert!(keys.len() > 50, "{def}: {keys:?}");
            let mut compared = 0;
            for key in keys.iter().take(400) {
                let e = eager.sprite_indexed(*key);
                let l = lazy.sprite_indexed(*key);
                assert_eq!(e.as_ref().map(|i| (i.w, i.h)), l.as_ref().map(|i| (i.w, i.h)), "{def} pal {pal} {key:?}: size");
                assert_eq!(e.as_ref().map(|i| i.pixels.as_slice()), l.as_ref().map(|i| i.pixels.as_slice()), "{def} pal {pal} {key:?}: pixels");
                assert_eq!(e.as_ref().map(|i| i.palette.as_deref()), l.as_ref().map(|i| i.palette.as_deref()), "{def} pal {pal} {key:?}: palette");
                assert_eq!(e.as_ref().map(|i| i.pal), l.as_ref().map(|i| i.pal), "{def} pal {pal} {key:?}: palette key");
                compared += 1;
            }
            assert!(compared > 50);
            assert_eq!(eager.rgba_sprite((9000, 0)).map(|i| i.rgba.clone()), lazy.rgba_sprite((9000, 0)).map(|i| i.rgba.clone()), "{def} pal {pal}: portrait");
        }
    }
}

#[test]
fn the_project_setting_keeps_small_files_eager() {
    let vfs = vfs();
    let def = find_char_defs(&vfs).into_iter().next().unwrap();
    let d = load_mugen_char_with(&vfs, &def, 1, true, LAZY_SFF_BYTES).unwrap();
    assert!(!d.is_lazy(), "bundled characters are small enough to decode up front");
}
