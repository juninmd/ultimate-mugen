use std::sync::Arc;
use std::time::Instant;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_zip, find_char_defs, resolve, text, Vfs};

fn bench(name: &str, iters: usize, mut f: impl FnMut()) {
    for _ in 0..(iters / 10).max(1) {
        f();
    }
    let mut samples: Vec<f64> = (0..iters)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64() * 1e6
        })
        .collect();
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    println!("{name:<28} mean {mean:>10.1} us   p50 {:>10.1}   p99 {:>10.1}", samples[samples.len() / 2], samples[samples.len() * 99 / 100]);
}

fn main() {
    let mut v = Vfs::new();
    add_zip(&mut v, include_bytes!("../../../assets/mugen/elecbyte.zip"), "").unwrap();
    let def = find_char_defs(&v).into_iter().next().unwrap();
    let dir = def.rsplit_once('/').map(|x| x.0).unwrap_or("");
    let sff = resolve(&v, dir, &format!("{}.sff", def.rsplit('/').next().unwrap().trim_end_matches(".def"))).map(|b| b.to_vec());
    if let Some(sff) = sff {
        bench("sff decode", 20, || {
            std::hint::black_box(ume_formats::sff::decode_sff(&sff).unwrap());
        });
    }
    let air = v.iter().find(|(k, _)| k.ends_with(".air")).map(|(_, b)| text(&b)).unwrap();
    bench("air parse", 50, || {
        std::hint::black_box(ume_formats::air::parse_air(&air));
    });
    let cns = v.iter().filter(|(k, _)| k.ends_with(".cns")).max_by_key(|(_, b)| b.len()).map(|(_, b)| text(&b)).unwrap();
    bench("cns parse+compile", 50, || {
        let mut m = std::collections::HashMap::new();
        ume_engine::cns::parse_states(&cns, &mut m, true);
        std::hint::black_box(m);
    });
    bench("char load (no images)", 10, || {
        std::hint::black_box(load_mugen_char(&v, &def, 1, false).unwrap());
    });
    let d = Arc::new(load_mugen_char(&v, &def, 1, false).unwrap());
    for n in [2usize, 6, 10] {
        let mut m = Match::new(common_def(), None, flat_stage());
        m.team_simul = true;
        for k in 0..n {
            m.add_player(d.clone(), k % 2, 8, 1 + k as i32 % 4);
        }
        m.start_round();
        let inputs = vec![0u32; n];
        let mut frame = 0u32;
        bench(&format!("match tick ({n} fighters)"), 2000, || {
            frame += 1;
            let mut inp = inputs.clone();
            inp[0] = if frame % 40 < 20 { 0 } else { 1 << (frame / 40 % 6) };
            m.tick(&inp);
        });
    }
}
