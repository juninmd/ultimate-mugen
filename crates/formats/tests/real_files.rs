//! Integration tests with real files (Ikemen GO screenpack + test-assets/kfm.zip).
//! Skipped (pass trivially) when the files are not present.

use std::path::{Path, PathBuf};
use std::time::Instant;
use ume_formats::act::decode_act;
use ume_formats::air::parse_air;
use ume_formats::ini::{find_section, parse_ini};
use ume_formats::sff::{decode_sff, sprite_to_rgba};
use ume_formats::snd::decode_snd;
use ume_formats::stage::load_stage;
use ume_formats::vfs::{add_dir, add_zip, find_char_defs, find_stage_defs, resolve, text, Vfs};

const SP: &str =
    "/tmp/claude-0/-home-user-ultimate-mugen/fa14b17d-4289-5fed-86c2-aacfb04ddae6/scratchpad/sp";

fn sp() -> Option<PathBuf> {
    let p = PathBuf::from(SP);
    if p.is_dir() {
        Some(p)
    } else {
        eprintln!("skipping: {SP} not found");
        None
    }
}

fn read(p: &Path) -> Vec<u8> {
    std::fs::read(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn kfm_zip() -> Option<Vec<u8>> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-assets/kfm.zip");
    std::fs::read(p).ok()
}

#[test]
fn kfm_sff_sprite_0_0() {
    let Some(sp) = sp() else { return };
    let buf = read(&sp.join("chars/kfm/kfm.sff"));
    let t = Instant::now();
    let sff = decode_sff(&buf).unwrap();
    eprintln!(
        "kfm.sff: {} sprites decoded in {:?}",
        sff.sprites.len(),
        t.elapsed()
    );
    assert_eq!(sff.version, 2);
    assert!(sff.sprites.len() > 100);
    assert!(!sff.palettes.is_empty());
    let s = sff
        .sprites
        .iter()
        .find(|s| s.group == 0 && s.index == 0)
        .unwrap();
    assert_eq!((s.width, s.height, s.axis_x, s.axis_y), (47, 106, 18, 105));
    let idx = s.indexed.as_ref().unwrap();
    assert_eq!(idx.len(), 47 * 106);
    assert!(s.palette.is_some());
    let opaque = idx.iter().filter(|&&v| v != 0).count() as f64 / idx.len() as f64;
    assert!(opaque > 0.2 && opaque < 0.9, "opaque ratio {opaque}");
    assert_eq!(idx[0], 0);
    let rgba = sprite_to_rgba(s, None);
    assert_eq!(rgba.len(), 47 * 106 * 4);
    assert_eq!(rgba[3], 0);
    assert!(
        rgba.as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] == 255)
            .count() as f64
            / (47.0 * 106.0)
            > 0.2
    );
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("sff")) {
            out.push(p);
        }
    }
}

#[test]
fn every_sff_decodes_consistently() {
    let Some(sp) = sp() else { return };
    let mut files = Vec::new();
    for d in ["chars", "stages", "data", "font"] {
        walk(&sp.join(d), &mut files);
    }
    files.sort();
    assert!(!files.is_empty());
    let mut problems = Vec::new();
    let t = Instant::now();
    for f in &files {
        let buf = read(f);
        let sff = match decode_sff(&buf) {
            Ok(s) => s,
            Err(e) => {
                problems.push(format!("{}: {e}", f.display()));
                continue;
            }
        };
        if sff.sprites.is_empty() {
            problems.push(format!("{}: no sprites", f.display()));
        }
        for s in &sff.sprites {
            let id = format!("{} {},{}", f.display(), s.group, s.index);
            let n = s.width as usize * s.height as usize;
            if s.indexed.as_ref().is_some_and(|v| v.len() != n) {
                problems.push(format!("{id}: indexed size"));
            }
            if s.rgba.as_ref().is_some_and(|v| v.len() != n * 4) {
                problems.push(format!("{id}: rgba size"));
            }
            if s.indexed.is_some() && n > 0 && s.palette.is_none() {
                problems.push(format!("{id}: no palette"));
            }
        }
        if sff.version == 2 {
            // recorded uncompressed sizes must match the sprite size
            let u32_at = |o: usize| u32::from_le_bytes(buf[o..o + 4].try_into().unwrap()) as usize;
            let u16_at = |o: usize| u16::from_le_bytes(buf[o..o + 2].try_into().unwrap()) as usize;
            let (so, ldata, tdata) = (u32_at(36), u32_at(52), u32_at(60));
            for (i, s) in sff.sprites.iter().enumerate() {
                let o = so + i * 28;
                let (fmt, len) = (buf[o + 14], u32_at(o + 20));
                if len != 0 && (2..=4).contains(&fmt) {
                    let start = (if u16_at(o + 26) & 1 != 0 {
                        tdata
                    } else {
                        ldata
                    }) + u32_at(o + 16);
                    if u32_at(start) != s.width as usize * s.height as usize {
                        problems.push(format!(
                            "{} {},{}: length prefix",
                            f.display(),
                            s.group,
                            s.index
                        ));
                    }
                }
            }
        }
    }
    eprintln!("{} SFF files decoded in {:?}", files.len(), t.elapsed());
    assert!(problems.is_empty(), "{problems:#?}");
}

#[test]
fn kfm_intro_sff_v1() {
    let Some(sp) = sp() else { return };
    let p = sp.join("chars/kfm/intro.sff");
    if !p.exists() {
        return;
    }
    let sff = decode_sff(&read(&p)).unwrap();
    assert_eq!(sff.version, 1);
    for s in &sff.sprites {
        assert!(s.width > 0);
        assert!(s.palette.is_some());
    }
}

#[test]
fn kfm_snd_wav_entries() {
    let Some(sp) = sp() else { return };
    let m = decode_snd(&read(&sp.join("chars/kfm/kfm.snd")));
    assert!(!m.is_empty());
    for v in m.values() {
        assert_eq!(&v[0..4], b"RIFF");
        assert_eq!(&v[8..12], b"WAVE");
    }
    assert!(m.contains_key(&(0, 0)));
}

#[test]
fn act_matches_sff_palette_table() {
    let Some(sp) = sp() else { return };
    let (act, sffp) = (
        sp.join("work/gofx/5300-0.act"),
        sp.join("data/gofx/gofx.sff"),
    );
    if !act.exists() || !sffp.exists() {
        return;
    }
    let pal = decode_act(&read(&act));
    let sff = decode_sff(&read(&sffp)).unwrap();
    let r = &sff
        .palettes
        .iter()
        .find(|p| p.group == 5300 && p.index == 0)
        .unwrap()
        .data;
    for c in 1..256 {
        assert_eq!(&pal[c * 4..c * 4 + 3], &r[c * 4..c * 4 + 3]);
    }
}

fn stage_vfs(sp: &Path) -> Vfs {
    let mut vfs = Vfs::new();
    add_dir(&mut vfs, &sp.join("stages"), "stages").unwrap();
    vfs
}

#[test]
fn stage0_loads() {
    let Some(sp) = sp() else { return };
    let vfs = stage_vfs(&sp);
    assert!(find_stage_defs(&vfs).contains(&"stages/stage0.def".to_string()));
    let t = Instant::now();
    let st = load_stage(&vfs, "stages/stage0.def").unwrap();
    eprintln!(
        "stage0: {} layers, {} sprites in {:?}",
        st.layers.len(),
        st.sprites.len(),
        t.elapsed()
    );
    assert_eq!(st.name, "Training Room");
    assert_eq!(st.local_w, 320.0);
    assert_eq!(
        (st.bound_left, st.bound_right, st.bound_high, st.bound_low),
        (-125.0, 125.0, -25.0, 0.0)
    );
    assert!((st.vertical_follow - 0.2).abs() < 1e-6);
    assert!(!st.layers.is_empty());
    for l in &st.layers {
        if l.anim.is_none() {
            let img = st
                .sprites
                .get(&l.sprite)
                .unwrap_or_else(|| panic!("missing sprite {:?}", l.sprite));
            assert_eq!(img.rgba.len(), (img.w * img.h * 4) as usize);
        }
    }
}

#[test]
fn kfm_stage_loads() {
    let Some(sp) = sp() else { return };
    let vfs = stage_vfs(&sp);
    let st = load_stage(&vfs, "stages/kfm.def").unwrap();
    eprintln!(
        "kfm stage: {:?} {} layers {} sprites",
        st.name,
        st.layers.len(),
        st.sprites.len()
    );
    assert!(!st.layers.is_empty());
    assert!(!st.sprites.is_empty());
    for l in &st.layers {
        if l.kind == "anim" {
            let a = l.anim.as_ref().expect("anim layer has its action");
            assert!(!a.frames.is_empty());
        } else {
            assert!(
                st.sprites.contains_key(&l.sprite),
                "missing sprite {:?}",
                l.sprite
            );
        }
    }
    // every stage def in the pack loads
    for d in find_stage_defs(&vfs) {
        load_stage(&vfs, &d).unwrap_or_else(|e| panic!("{d}: {e}"));
    }
    // 720p stage reports its localcoord
    if vfs.contains_key("stages/stage0-720.def") {
        let st = load_stage(&vfs, "stages/stage0-720.def").unwrap();
        assert_eq!(st.local_w, 1280.0);
    }
}

#[test]
fn chars_dir_discovery() {
    let Some(sp) = sp() else { return };
    let mut vfs = Vfs::new();
    let n = add_dir(&mut vfs, &sp.join("chars"), "chars").unwrap();
    assert!(n > 10);
    let defs = find_char_defs(&vfs);
    assert!(defs.contains(&"chars/kfm/kfm.def".to_string()), "{defs:?}");
    assert!(
        defs.contains(&"chars/kfm720/kfm720.def".to_string())
            || defs.iter().any(|d| d.starts_with("chars/kfm720/")),
        "{defs:?}"
    );
    assert!(!defs
        .iter()
        .any(|d| d.ends_with("intro.def") || d.ends_with("ending.def")));
}

#[test]
fn kfm_zip_loads() {
    let Some(zip) = kfm_zip() else {
        eprintln!("skipping: test-assets/kfm.zip not found");
        return;
    };
    let mut vfs = Vfs::new();
    let n = add_zip(&mut vfs, &zip, "").unwrap();
    assert_eq!(n, 12);
    let defs = find_char_defs(&vfs);
    assert_eq!(defs, vec!["kfm/kfm.def"]);
    let secs = parse_ini(&vfs.get("kfm/kfm.def").map(|b| text(&b)).unwrap_or_default());
    let files = find_section(&secs, "files").unwrap();
    let t = Instant::now();
    let sff = decode_sff(&resolve(&vfs, "kfm", files.get("sprite").unwrap()).unwrap()).unwrap();
    let anims = parse_air(&text(
        resolve(&vfs, "kfm", files.get("anim").unwrap()).unwrap(),
    ));
    let snd = decode_snd(&resolve(&vfs, "kfm", files.get("sound").unwrap()).unwrap());
    eprintln!(
        "kfm from zip: {} sprites, {} anims, {} sounds in {:?}",
        sff.sprites.len(),
        anims.len(),
        snd.len(),
        t.elapsed()
    );
    assert!(sff
        .sprites
        .iter()
        .any(|s| s.group == 0 && s.index == 0 && s.width == 47 && s.height == 106));
    let a0 = &anims[&0];
    assert!(!a0.frames.is_empty());
    assert!(!a0.frames[0].clsn2.is_empty());
    assert!(anims.len() > 50);
    assert!(!snd.is_empty());
    assert!(resolve(&vfs, "kfm", files.get("cns").unwrap()).is_some());
    assert!(resolve(&vfs, "kfm", files.get("cmd").unwrap()).is_some());
}
