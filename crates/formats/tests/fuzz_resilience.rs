use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::time::{Duration, Instant};
use ume_formats::air::parse_air;
use ume_formats::fnt::decode_fnt;
use ume_formats::ini::parse_ini;
use ume_formats::pcx::decode_pcx;
use ume_formats::png::decode_png;
use ume_formats::sff::{decode_lz5, decode_rle5, decode_rle8, decode_sff};
use ume_formats::snd::{decode_snd, decode_snd_checked};
use ume_formats::vfs::{add_zip, Vfs};

const MAX_DECODE_TIME: Duration = Duration::from_secs(5);

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn mutate(src: &[u8], rng: &mut XorShift) -> Vec<u8> {
    let mut out = src.to_vec();
    if out.is_empty() {
        return out;
    }
    match rng.below(5) {
        0 => {
            for _ in 0..1 + rng.below(16) {
                let i = rng.below(out.len());
                out[i] ^= 1 << rng.below(8);
            }
        }
        1 => out.truncate(rng.below(out.len())),
        2 => {
            let i = rng.below(out.len());
            let end = (i + 1 + rng.below(64)).min(out.len());
            for b in &mut out[i..end] {
                *b = 0xFF;
            }
        }
        3 => {
            let i = rng.below(out.len().saturating_sub(4).max(1));
            let end = (i + 4).min(out.len());
            for b in &mut out[i..end] {
                *b = [0x00, 0xFF, 0x7F, 0x80][rng.below(4)];
            }
        }
        _ => {
            let i = rng.below(out.len());
            let junk: Vec<u8> = (0..rng.below(256)).map(|_| rng.next() as u8).collect();
            out.splice(i..i, junk);
        }
    }
    out
}

fn survives(name: &str, input: &[u8], run: impl Fn(&[u8])) {
    let start = Instant::now();
    let result = catch_unwind(AssertUnwindSafe(|| run(input)));
    assert!(
        result.is_ok(),
        "{name} panicked on {} byte input",
        input.len()
    );
    assert!(
        start.elapsed() < MAX_DECODE_TIME,
        "{name} took {:?} on {} byte input",
        start.elapsed(),
        input.len()
    );
}

fn mutations_per_file() -> usize {
    std::env::var("FUZZ_ITERATIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(400)
}

fn fuzz_corpus(name: &str, corpus: &[Vec<u8>], run: impl Fn(&[u8])) {
    let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
    for seed in corpus {
        survives(name, seed, &run);
        for _ in 0..mutations_per_file() {
            survives(name, &mutate(seed, &mut rng), &run);
        }
    }
}

fn corpus_from_zip(ext: &str) -> Vec<Vec<u8>> {
    let zip = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/mugen/elecbyte.zip");
    let Ok(bytes) = std::fs::read(zip) else {
        return Vec::new();
    };
    let mut vfs = Vfs::new();
    add_zip(&mut vfs, &bytes, "").unwrap();
    let mut files: Vec<_> = vfs
        .iter()
        .filter(|(k, _)| k.ends_with(ext))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    files.sort_by_key(|a| a.1.len());
    files.into_iter().take(3).map(|(_, v)| v).collect()
}

fn random_corpus() -> Vec<Vec<u8>> {
    let mut rng = XorShift(42);
    (0..8)
        .map(|n| (0..16 << n).map(|_| rng.next() as u8).collect())
        .collect()
}

#[test]
fn sff_decoder_survives_mutations() {
    let mut corpus = corpus_from_zip(".sff");
    corpus.extend(random_corpus());
    fuzz_corpus("decode_sff", &corpus, |b| {
        let _ = decode_sff(b);
    });
}

#[test]
fn snd_decoder_survives_mutations() {
    let mut corpus = corpus_from_zip(".snd");
    corpus.extend(random_corpus());
    fuzz_corpus("decode_snd", &corpus, |b| {
        let _ = decode_snd(b);
        let _ = decode_snd_checked(b);
    });
}

#[test]
fn raster_decoders_survive_mutations() {
    let corpus = random_corpus();
    fuzz_corpus("pcx/png/fnt", &corpus, |b| {
        let _ = decode_pcx(b, true);
        let _ = decode_png(b);
        let _ = decode_fnt(b);
    });
}

#[test]
fn rle_decoders_survive_hostile_sizes() {
    for seed in random_corpus() {
        for size in [0usize, 1, 4096, 1 << 20] {
            survives("rle", &seed, |b| {
                let _ = decode_rle8(b, size);
                let _ = decode_rle5(b, size);
                let _ = decode_lz5(b, size);
            });
        }
    }
}

#[test]
fn text_parsers_survive_mutations() {
    let corpus: Vec<Vec<u8>> = corpus_from_zip(".air")
        .into_iter()
        .chain(corpus_from_zip(".def"))
        .chain(random_corpus())
        .collect();
    fuzz_corpus("parse_ini/air", &corpus, |b| {
        let text = String::from_utf8_lossy(b);
        let _ = parse_ini(&text);
        let _ = parse_air(&text);
    });
}
