// Embeds every UMF character JSON in assets/chars/ (order: index.json first, then the rest alphabetically).
use std::{env, fs, path::Path};
fn main() {
    // the release tag is the version source of truth: CI exports it, local builds are marked `-dev`
    println!("cargo:rerun-if-env-changed=UME_RELEASE_VERSION");
    let version = env::var("UME_RELEASE_VERSION")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| format!("{}-dev", env::var("CARGO_PKG_VERSION").unwrap()));
    println!("cargo:rustc-env=UME_VERSION={version}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    let hash = std::process::Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    println!("cargo:rustc-env=UME_GIT_HASH={hash}");
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/chars");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut ids: Vec<String> = fs::read_to_string(dir.join("index.json"))
        .ok()
        .map(|s| {
            let s = s
                .trim()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string();
            s.split(',')
                .map(|x| x.trim().trim_matches('"').to_string())
                .filter(|x| !x.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let mut rest: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".json") && n != "index.json")
        .map(|n| n.trim_end_matches(".json").to_string())
        .filter(|id| !ids.contains(id))
        .collect();
    rest.sort();
    ids.retain(|id| dir.join(format!("{id}.json")).exists());
    ids.extend(rest);
    let mut out = String::from("pub const BUILTIN: &[(&str, &str)] = &[\n");
    for id in &ids {
        let p = dir.join(format!("{id}.json"));
        println!("cargo:rerun-if-changed={}", p.display());
        out += &format!(
            "    ({id:?}, include_str!({:?})),\n",
            p.display().to_string()
        );
    }
    out += "];\n";
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("chars_gen.rs"),
        out,
    )
    .unwrap();
}
