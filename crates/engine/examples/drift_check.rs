use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

fn main() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../mugen/chars");
    let mut dirs: Vec<_> = std::fs::read_dir(root).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    for dir in dirs {
        let mut vfs = Vfs::new();
        if add_dir(&mut vfs, &dir, "c").is_err() { continue; }
        let Some(def) = find_char_defs(&vfs).into_iter().next() else { continue };
        let Ok(a) = load_mugen_char(&vfs, &def, 1, false) else { continue };
        let Ok(b) = load_mugen_char(&vfs, &def, 2, false) else { continue };
        let (a, b) = (Arc::new(a), Arc::new(b));
        let mut m = Match::new(common_def(), None, flat_stage());
        m.add_player(a, 0, 0, 1);
        m.add_player(b, 1, 0, 2);
        m.start_round();
        m.intro_ticks = 170;
        let mut worst = 0.0f32;
        let mut state = 0;
        for _ in 0..600 {
            m.tick(&[0, 0]);
            let f = &m.fighters[0];
            if f.state_no != 0 && f.state_no != 191 && f.state_no != 192 && worst < 1.0 {
                state = f.state_no;
            }
            worst = worst.max((f.pos.x + 70.0).abs());
        }
        if worst > 5.0 {
            println!("MOVES {worst:.1}px state={state} {}", dir.file_name().unwrap().to_string_lossy());
        }
    }
    println!("done");
}
