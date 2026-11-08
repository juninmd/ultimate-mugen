//! Lists M.U.G.E.N characters the catalogue would quarantine (missing files named by their `.def`).
//! Usage: cargo run --release -p ume-engine --example char_health -- <char_dir>...
use std::collections::BTreeMap;
use ume_formats::health::problems;
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

fn main() {
    let (mut healthy, mut broken) = (0usize, 0usize);
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    for dir in std::env::args().skip(1) {
        let mut vfs = Vfs::new();
        if add_dir(&mut vfs, std::path::Path::new(&dir), "c").is_err() {
            continue;
        }
        for def in find_char_defs(&vfs) {
            let found = problems(&vfs, &def);
            if found.is_empty() {
                healthy += 1;
                if std::env::var_os("HEALTHY").is_some() {
                    println!("OK {def}");
                }
                continue;
            }
            broken += 1;
            println!("BROKEN {def}: {}", found.join("; "));
            for p in found {
                let key = p.split(" '").next().unwrap_or(&p).to_string();
                *reasons.entry(key).or_default() += 1;
            }
        }
    }
    println!("healthy={healthy} broken={broken}");
    for (reason, n) in reasons {
        println!("  {n:4} x {reason}");
    }
}
