//! Audits MUGEN characters for animation frames whose sprite is missing or decoded empty.
//! Usage: cargo run --release -p ume-engine --example sprite_audit -- <char_dir>...
use std::collections::BTreeSet;
use ume_engine::loader::load_mugen_char;
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

fn main() {
    for dir in std::env::args().skip(1) {
        let mut vfs = Vfs::new();
        if add_dir(&mut vfs, std::path::Path::new(&dir), "c").is_err() { continue; }
        for def in find_char_defs(&vfs) {
            let d = match load_mugen_char(&vfs, &def, 1, true) { Ok(d) => d, Err(e) => { println!("FAIL {def}: {e}"); continue; } };
            let (mut frames, mut missing, mut empty, mut blank) = (0, BTreeSet::new(), BTreeSet::new(), 0);
            for (no, a) in &d.anims {
                for fr in &a.frames {
                    frames += 1;
                    let key = (fr.group, fr.index);
                    match d.sprites.get(&key) {
                        None => { missing.insert((*no, fr.group, fr.index)); }
                        Some(s) if s.width == 0.0 || s.height == 0.0 => { empty.insert((*no, fr.group, fr.index)); }
                        Some(s) => {
                            let blank_sprite = match d.indexed.get(&key) {
                                Some(ix) => ix.pixels.iter().all(|&p| p == 0),
                                None => s.image.as_ref().is_none_or(|i| i.rgba.chunks(4).all(|p| p[3] == 0)),
                            };
                            if blank_sprite { blank += 1; if blank <= 3 { println!("   blank sprite {},{} {}x{} (anim {no})", fr.group, fr.index, s.width, s.height); } }
                        }
                    }
                }
            }
            let anims_missing: BTreeSet<i32> = missing.iter().map(|m| m.0).collect();
            println!("{def}: frames={frames} missing={} (anims {}) noimage={} fully-transparent={blank} e.g. {:?}", missing.len(), anims_missing.len(), empty.len(), missing.iter().take(4).collect::<Vec<_>>());
        }
    }
}
