use crate::chardef::default_constants;
use crate::cns::{parse_constants, scale_default_constants};
use std::collections::HashSet;
use ume_formats::ini::{find_section, parse_ini_no_lines};
use ume_formats::vfs::{norm_path, resolve, text, Vfs};

use super::format::local_size;

pub fn load_mugen_stats(vfs: &Vfs, def_path: &str) -> Option<(f64, f64, f64)> {
    let def_path = norm_path(def_path);
    let base = def_path
        .rsplit_once('/')
        .map(|(b, _)| b.to_string())
        .unwrap_or_default();
    let dt = text(vfs.get(&def_path)?);
    let secs = parse_ini_no_lines(&dt);
    let info = find_section(&secs, "info");
    let files = find_section(&secs, "files")?;
    let (local_w, _) = local_size(info.and_then(|s| s.get("localcoord")));
    let cns_name = files.get("cns")?;
    let cns_buf = resolve(vfs, &base, cns_name)?;
    let cns_text = text(cns_buf);
    let mut constants = default_constants();
    let mut provided = HashSet::new();
    parse_constants(&cns_text, &mut constants, &mut provided);
    scale_default_constants(&mut constants, &provided, local_w as f64 / 320.0);
    let attack = constants.get("data.attack").copied().unwrap_or(100.0);
    let life = constants.get("data.life").copied().unwrap_or(1000.0);
    let speed = constants.get("velocity.walk.fwd.x").copied().unwrap_or(2.4);
    Some((attack, life, speed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_mugen_stats_reads_attack_life_speed_without_decoding_sprites() {
        let mut vfs = Vfs::new();
        let def = "[Info]\nname = \"Test\"\n[Files]\ncns = test.cns\n";
        let cns = "[Data]\nlife = 1200\nattack = 110\n[Velocity]\nwalk.fwd = 3.2\n";
        vfs.insert("chars/test/test.def".into(), def.as_bytes().to_vec());
        vfs.insert("chars/test/test.cns".into(), cns.as_bytes().to_vec());
        let stats = load_mugen_stats(&vfs, "chars/test/test.def").expect("stats");
        assert_eq!(stats, (110.0, 1200.0, 3.2));
    }
}
