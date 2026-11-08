use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use ume_formats::vfs::{add_zip, text, Vfs};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }
}

fn seed_vfs() -> Option<Vfs> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/mugen/elecbyte.zip");
    let bytes = std::fs::read(p).ok()?;
    let mut vfs = Vfs::new();
    add_zip(&mut vfs, &bytes, "").ok()?;
    Some(vfs)
}

fn seeds(vfs: &Vfs, ext: &str, max_len: usize) -> Vec<Vec<u8>> {
    let mut v: Vec<(String, Vec<u8>)> = vfs
        .iter()
        .filter(|(k, b)| k.to_lowercase().ends_with(ext) && b.len() <= max_len)
        .map(|(k, b)| (k.clone(), b.to_vec()))
        .collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v.into_iter().map(|x| x.1).collect()
}

const INTERESTING: [u8; 8] = [0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff, 0x40, 0xc0];

fn mutate(r: &mut Rng, src: &[u8]) -> Vec<u8> {
    let mut b = src.to_vec();
    for _ in 0..1 + r.below(8) {
        if b.is_empty() {
            b.push(r.next() as u8);
            continue;
        }
        let len = b.len();
        match r.below(7) {
            0 => {
                let i = r.below(len);
                b[i] ^= 1 << r.below(8);
            }
            1 => {
                let i = r.below(len);
                b[i] = INTERESTING[r.below(INTERESTING.len())];
            }
            2 => {
                let n = r.below(len);
                b.truncate(n);
            }
            3 => {
                let i = r.below(len);
                let n = r.below(64).min(len - i);
                b.drain(i..i + n);
            }
            4 => {
                let i = r.below(len);
                let n = r.below(32);
                let s: Vec<u8> = (0..n).map(|_| r.next() as u8).collect();
                b.splice(i..i, s);
            }
            5 => {
                let i = r.below(len.saturating_sub(3).max(1));
                for k in 0..4.min(len - i) {
                    b[i + k] = INTERESTING[r.below(INTERESTING.len())];
                }
            }
            _ => {
                let i = r.below(len);
                let n = r.below(256).min(len - i);
                let j = r.below(len);
                let chunk: Vec<u8> = b[i..i + n].to_vec();
                for (k, c) in chunk.into_iter().enumerate() {
                    if j + k < len {
                        b[j + k] = c;
                    }
                }
            }
        }
    }
    b
}

fn run(name: &str, corpus: &[Vec<u8>], iters: usize, target: impl Fn(&[u8])) {
    assert!(!corpus.is_empty(), "no seeds for {name}");
    let mut r = Rng(0x9e37_79b9_7f4a_7c15);
    let budget = Duration::from_millis(1500);
    let mut failures = Vec::new();
    for i in 0..iters {
        let src = &corpus[r.below(corpus.len())];
        let input = mutate(&mut r, src);
        let t = Instant::now();
        let ok = catch_unwind(AssertUnwindSafe(|| target(&input))).is_ok();
        let el = t.elapsed();
        if !ok || el > budget {
            let path = std::env::temp_dir().join(format!("fuzz-{name}-{i}.bin"));
            let _ = std::fs::write(&path, &input);
            failures.push(format!("iter {i}: {} ({} ms) -> {}", if ok { "slow" } else { "panic" }, el.as_millis(), path.display()));
            if failures.len() >= 5 {
                break;
            }
        }
    }
    assert!(failures.is_empty(), "{name}: {}", failures.join("\n"));
}

fn iters() -> usize {
    std::env::var("UME_FUZZ_ITERS").ok().and_then(|v| v.parse().ok()).unwrap_or(300)
}

#[test]
fn fuzz_sff() {
    let Some(v) = seed_vfs() else { return };
    let mut c = seeds(&v, ".sff", 200_000);
    for s in seeds(&v, ".sff", 2_000_000) {
        c.push(s.iter().take(60_000).copied().collect());
    }
    run("sff", &c, iters(), |b| {
        if let Ok(s) = ume_formats::sff::decode_sff(b) {
            for sp in s.sprites.iter().take(64) {
                let _ = ume_formats::sff::sprite_to_rgba(sp, None);
            }
        }
    });
}

#[test]
fn fuzz_air() {
    let Some(v) = seed_vfs() else { return };
    run("air", &seeds(&v, ".air", 1 << 20), iters(), |b| {
        let _ = ume_formats::air::parse_air(&text(b));
    });
}

#[test]
fn fuzz_ini_def() {
    let Some(v) = seed_vfs() else { return };
    let mut c = seeds(&v, ".def", 1 << 20);
    c.extend(seeds(&v, ".cns", 1 << 20));
    c.extend(seeds(&v, ".cmd", 1 << 20));
    run("ini", &c, iters(), |b| {
        let t = text(b);
        for s in ume_formats::ini::parse_ini(&t) {
            for l in s.lines.iter().take(500) {
                let _ = ume_formats::ini::split_top(l);
                let _ = ume_formats::ini::nums(Some(l), &[0.0]);
            }
        }
    });
}

#[test]
fn fuzz_snd_act_fnt_pcx_png() {
    let Some(v) = seed_vfs() else { return };
    let mut snd = seeds(&v, ".snd", 1 << 20);
    snd.push(b"ElecbyteSnd\0\x01\0\x01\0\x02\0\0\0\x01\0\0\0\x18\0\0\0".to_vec());
    run("snd", &snd, iters(), |b| {
        let _ = ume_formats::snd::decode_snd(b);
        let _ = ume_formats::snd::decode_snd_checked(b);
    });
    let act: Vec<Vec<u8>> = vec![vec![7; 768], vec![1; 100]];
    run("act", &act, iters(), |b| {
        let _ = ume_formats::act::decode_act(b);
    });
    let pcx = vec![{
        let mut h = vec![0x0a, 5, 1, 8, 0, 0, 0, 0, 15, 0, 15, 0, 0, 0, 0, 0];
        h.resize(65, 0);
        h.extend_from_slice(&[1, 16, 0, 1, 0]);
        h.resize(128, 0);
        h.extend((0..400).map(|i| (i * 7) as u8));
        h
    }];
    run("pcx", &pcx, iters(), |b| {
        let _ = ume_formats::pcx::decode_pcx(b, true);
    });
    let png: Vec<Vec<u8>> = vec![vec![0x89, b'P', b'N', b'G', 13, 10, 26, 10, 0, 0, 0, 13, b'I', b'H', b'D', b'R', 0, 0, 0, 4, 0, 0, 0, 4, 8, 3, 0, 0, 0]];
    run("png", &png, iters(), |b| {
        let _ = ume_formats::png::decode_png(b);
    });
}

#[test]
fn fuzz_zip_and_stage() {
    let Some(v) = seed_vfs() else { return };
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/mugen/elecbyte.zip");
    let zip = std::fs::read(p).unwrap();
    let mut small = zip.clone();
    small.drain(..zip.len().saturating_sub(40_000).min(zip.len() / 2));
    run("zip", &[small, zip.iter().rev().take(70_000).rev().copied().collect()], iters(), |b| {
        let mut vf = Vfs::new();
        let _ = add_zip(&mut vf, b, "");
    });
    let defs: Vec<Vec<u8>> = seeds(&v, ".def", 1 << 20);
    run("stage", &defs, iters(), |b| {
        let mut vf = Vfs::new();
        for (k, bytes) in v.iter() {
            if k.starts_with("stages/") && k.ends_with(".sff") {
                vf.insert(k.clone(), bytes.to_vec());
            }
        }
        vf.insert("stages/fuzz.def".into(), b.to_vec());
        let _ = ume_formats::stage::load_stage(&vf, "stages/fuzz.def");
    });
}
