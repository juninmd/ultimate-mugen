use std::path::Path;
use ume_formats::screenpack::load_screenpack;
use ume_formats::vfs::{add_dir, Vfs};

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let system = std::env::args().nth(2).unwrap_or_else(|| "data/system.def".into());
    let mut vfs = Vfs::new();
    if let Err(e) = add_dir(&mut vfs, Path::new(&root), "") {
        eprintln!("{root}: {e}");
        std::process::exit(2);
    }
    match load_screenpack(&vfs, &system) {
        Ok(sp) => {
            println!("menu items: {}", sp.system.menu.len());
            println!("select slots: {}", sp.select.slots.len());
            println!("fonts: {} system, {} fight", sp.fonts.len(), sp.fight_fonts.len());
            println!("sprites: {}", sp.sprites.as_ref().map_or(0, |s| s.sprites.len()));
            println!("sounds: {}", sp.sounds.len());
            println!("storyboards: logo={} intro={}", sp.logo.is_some(), sp.intro.is_some());
            for w in &sp.warnings {
                println!("warning: {w}");
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
