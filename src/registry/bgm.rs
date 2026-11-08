use super::MusicLoop;
use ume_formats::ini::{find_section, parse_ini, unquote};
use ume_formats::vfs::{text, Vfs};
pub fn stage_bgm(vfs: &Vfs, def: &str) -> Option<(Vec<u8>, MusicLoop)> {
    let secs = parse_ini(&text(vfs.get(&ume_formats::vfs::norm_path(def))?));
    let m = find_section(&secs, "music")?;
    let name = unquote(m.get("bgmusic")?);
    if name.is_empty() {
        return None;
    }
    let base = def.rfind('/').map(|i| &def[..i]).unwrap_or("");
    let bytes = ume_formats::vfs::resolve(vfs, base, &name)
        .or_else(|| ume_formats::vfs::resolve(vfs, "", &name))?;
    let n = |k: &str| {
        m.get(k)
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(0)
    };
    Some((
        bytes.to_vec(),
        MusicLoop {
            start: n("bgmloopstart"),
            end: n("bgmloopend"),
        },
    ))
}
