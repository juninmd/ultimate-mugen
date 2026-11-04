//! The metadata-only pass and single-sprite decoding must agree with a full decode.

use ume_formats::ini::{find_section, parse_ini};
use ume_formats::sff::{decode_sff, decode_sff_metadata, decode_sff_sprites};
use ume_formats::vfs::{add_zip, find_char_defs, resolve, text, Vfs};

fn zip_bytes() -> Option<Vec<u8>> {
    std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/mugen/elecbyte.zip")).ok()
}

#[test]
fn metadata_pass_matches_the_full_decode() {
    let Some(zip) = zip_bytes() else { eprintln!("skipping: elecbyte.zip not found"); return };
    let mut vfs = Vfs::new();
    add_zip(&mut vfs, &zip, "").unwrap();
    let mut checked = 0;
    for def in find_char_defs(&vfs) {
        let base = def.rsplit_once('/').map(|(b, _)| b).unwrap_or("");
        let secs = parse_ini(&text(vfs.get(&def).unwrap()));
        let sprite = find_section(&secs, "files").and_then(|s| s.get("sprite").map(|v| v.to_string())).unwrap_or_default();
        let Some(buf) = resolve(&vfs, base, &sprite) else { continue };
        let eager = decode_sff(buf).unwrap();
        let meta = decode_sff_metadata(buf).unwrap();
        assert_eq!(eager.sprites.len(), meta.sprites.len(), "{def}");
        for (a, b) in eager.sprites.iter().zip(&meta.sprites) {
            assert_eq!((a.group, a.index, a.width, a.height, a.axis_x, a.axis_y), (b.group, b.index, b.width, b.height, b.axis_x, b.axis_y), "{def} metadata");
            assert!(b.indexed.is_none() && b.rgba.is_none(), "{def} metadata must not decode pixels");
        }
        let mut pick = 0;
        for s in eager.sprites.iter().filter(|s| s.indexed.is_some() || s.rgba.is_some()).step_by(37).take(25) {
            let one = decode_sff_sprites(buf, &[(s.group, s.index)]).unwrap();
            let got = one.sprites.iter().find(|d| d.group == s.group && d.index == s.index).expect("requested sprite decoded");
            assert_eq!((got.width, got.height), (s.width, s.height), "{def} {:?}", (s.group, s.index));
            assert_eq!(got.indexed.as_deref(), s.indexed.as_deref(), "{def} {:?} pixels", (s.group, s.index));
            assert_eq!(got.rgba.as_deref(), s.rgba.as_deref(), "{def} {:?} rgba", (s.group, s.index));
            assert_eq!(got.palette.as_deref(), s.palette.as_deref(), "{def} {:?} palette", (s.group, s.index));
            pick += 1;
        }
        checked += pick;
    }
    assert!(checked > 0, "no sprites checked");
}

#[test]
fn a_requested_sprite_pulls_its_link_source() {
    let Some(zip) = zip_bytes() else { eprintln!("skipping: elecbyte.zip not found"); return };
    let mut vfs = Vfs::new();
    add_zip(&mut vfs, &zip, "").unwrap();
    for def in find_char_defs(&vfs) {
        let base = def.rsplit_once('/').map(|(b, _)| b).unwrap_or("");
        let secs = parse_ini(&text(vfs.get(&def).unwrap()));
        let sprite = find_section(&secs, "files").and_then(|s| s.get("sprite").map(|v| v.to_string())).unwrap_or_default();
        let Some(buf) = resolve(&vfs, base, &sprite) else { continue };
        let eager = decode_sff(buf).unwrap();
        let linked = eager.sprites.iter().position(|s| s.indexed.is_none() && s.width > 0);
        let Some(at) = linked else { continue };
        let target = &eager.sprites[at];
        let one = decode_sff_sprites(buf, &[(target.group, target.index)]).unwrap();
        let got = one.sprites.iter().find(|d| d.group == target.group && d.index == target.index).unwrap();
        assert_eq!(got.indexed.as_deref(), target.indexed.as_deref(), "{def}: linked sprite keeps its pixels");
        return;
    }
}
