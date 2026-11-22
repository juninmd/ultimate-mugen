//! Build a CharDef from a MUGEN character inside a VFS.
use crate::chardef::CharDef;
use std::collections::HashMap;
use ume_formats::air::parse_air;
use ume_formats::ini::{find_section, parse_ini_no_lines, unquote_str};
use ume_formats::snd::decode_snd;
use ume_formats::vfs::{norm_path, resolve, text, Vfs};

mod assemble;
mod format;
mod movelist;
mod portrait;
mod sff_cache;
#[cfg(target_arch = "wasm32")] mod sff_slice;
mod sources;
mod sprites;
mod stats;
#[cfg(test)]
use format::local_size;
pub use {assemble::assemble_mugen_char, portrait::load_mugen_portrait, sources::{MugenSources, MugenText}, stats::load_mugen_stats};

pub fn load_mugen_char(vfs: &Vfs, def_path: &str, pal_no: i32, with_images: bool) -> Result<CharDef, String> {
    let def_path = norm_path(def_path);
    let src = read_mugen_sources(vfs, &def_path)?;
    let d = assemble_mugen_char(src, pal_no, with_images);
    report_diagnostics(&d, &def_path);
    Ok(d)
}

fn info_text(info: Option<&ume_formats::ini::Section>, key: &str) -> Option<String> {
    info.and_then(|s| s.get(key)).map(|v| unquote_str(v).to_string())
}

pub fn read_mugen_sources(vfs: &Vfs, def_path: &str) -> Result<MugenSources, String> {
    let def_path = norm_path(def_path);
    let base = def_path.rsplit_once('/').map(|(b, _)| b.to_string()).unwrap_or_default();
    let dt = text(vfs.get(&def_path).ok_or("def not found")?);
    let secs = parse_ini_no_lines(&dt);
    let info = find_section(&secs, "info");
    let files = find_section(&secs, "files");
    let fget = |k: &str| files.and_then(|s| s.get(k));
    let read = |name: &str| resolve(vfs, &base, name).map(text).unwrap_or_default();
    let name = info_text(info, "name").filter(|s| !s.is_empty()).unwrap_or_else(|| def_path.clone());
    let display_name = info_text(info, "displayname").filter(|s| !s.is_empty()).unwrap_or_else(|| name.clone());
    let common_name = unquote_str(fget("stcommon").unwrap_or_default()).to_string();
    let stcommon = (!common_name.is_empty() && !common_name.contains(".."))
        .then(|| resolve(vfs, &base, &common_name).map(|b| sources::MugenText::new(common_name.clone(), text(b))))
        .flatten();
    let cns_name = fget("cns").unwrap_or_default();
    let cns_text = crate::perf::time("cns:read", || fget("cns").map(&read).unwrap_or_default());
    let mut st = vec![];
    for k in ["st", "st0", "st1", "st2", "st3", "st4", "st5", "st6", "st7", "st8", "st9"] {
        let Some(fname) = fget(k) else { continue };
        if norm_path(unquote_str(fname)) == norm_path(unquote_str(cns_name)) {
            continue;
        }
        if let Some(b) = resolve(vfs, &base, fname) {
            st.push(sources::MugenText::new(fname, text(b)));
        }
    }
    let cmd_name = fget("cmd").unwrap_or_default();
    let cmd_text = fget("cmd").map(&read).unwrap_or_default();
    let sprite_buf = fget("sprite").and_then(|p| resolve(vfs, &base, p));
    let anim_buf = fget("anim").and_then(|p| resolve(vfs, &base, p));
    let snd_buf = fget("sound").and_then(|p| resolve(vfs, &base, p));
    let (sff, anims, sounds) = decode_binaries(sprite_buf, anim_buf, snd_buf);
    let pal_files = (1..=12).map(|k| fget(&format!("pal{k}")).and_then(|p| resolve(vfs, &base, p)).map(|b| b.to_vec())).collect();
    Ok(MugenSources {
        id: format!("mugen:{def_path}"),
        author: info_text(info, "author").unwrap_or_default(),
        localcoord: info.and_then(|s| s.get("localcoord")).map(str::to_string),
        mugen_version: info.and_then(|s| s.get("mugenversion")).map(str::to_string),
        name,
        display_name,
        stcommon,
        cns: sources::MugenText::new(cns_name, cns_text),
        st,
        cmd: sources::MugenText::new(cmd_name, cmd_text),
        sff: sff.transpose()?,
        anims,
        sounds,
        pal_files,
    })
}

type Binaries = (Option<Result<ume_formats::sff::DecodedSff, String>>, HashMap<i32, crate::chardef::Animation>, HashMap<(i32, i32), Vec<u8>>);

#[cfg(not(target_arch = "wasm32"))]
fn decode_binaries(sprite_buf: Option<impl AsRef<[u8]> + Send>, anim_buf: Option<impl AsRef<[u8]> + Send>, snd_buf: Option<impl AsRef<[u8]> + Send>) -> Binaries {
    std::thread::scope(|sc| {
        let h_sff = sc.spawn(move || sprite_buf.map(|buf| crate::perf::time("sff:decode", || sff_cache::decode_cached(buf.as_ref(), &std::collections::HashSet::new()))));
        let h_anim = sc.spawn(move || crate::perf::time("air:parse", || anim_buf.map(|b| parse_air(&text(b.as_ref()))).unwrap_or_default()));
        let h_snd = sc.spawn(move || crate::perf::time("snd:decode", || snd_buf.map(|b| decode_snd(b.as_ref())).unwrap_or_default()));
        (h_sff.join().unwrap(), h_anim.join().unwrap(), h_snd.join().unwrap())
    })
}

#[cfg(target_arch = "wasm32")]
fn decode_binaries(sprite_buf: Option<impl AsRef<[u8]>>, anim_buf: Option<impl AsRef<[u8]>>, snd_buf: Option<impl AsRef<[u8]>>) -> Binaries {
    let anims = crate::perf::time("air:parse", || anim_buf.map(|b| parse_air(&text(b.as_ref()))).unwrap_or_default());
    let sff = sprite_buf.map(|buf| crate::perf::time("sff:decode", || sff_slice::decode_for_anims(buf.as_ref(), &anims)));
    let sounds = crate::perf::time("snd:decode", || snd_buf.map(|b| decode_snd(b.as_ref())).unwrap_or_default());
    (sff, anims, sounds)
}

fn report_diagnostics(d: &CharDef, def_path: &str) {
    if cfg!(target_arch = "wasm32") || std::env::var_os("UME_QUIET_DIAGNOSTICS").is_some() {
        return;
    }
    eprint!(
        "{}",
        d.diagnostics()
            .report(def_path, std::env::var_os("TRACE").is_some())
    );
}

#[cfg(test)]
mod tests {
    use super::local_size;

    #[test]
    fn localcoord_accepts_two_values() {
        assert_eq!(local_size(Some("400, 300")), (400.0, 300.0));
    }

    #[test]
    fn localcoord_single_value_does_not_panic() {
        // chars de fullgames (KOF Zillion) usam `localcoord = 400`
        assert_eq!(local_size(Some("400")), (400.0, 240.0));
    }

    #[test]
    fn localcoord_missing_or_invalid_falls_back() {
        assert_eq!(local_size(None), (320.0, 240.0));
        assert_eq!(local_size(Some("")), (320.0, 240.0));
        assert_eq!(local_size(Some("0")), (320.0, 240.0));
    }
}
