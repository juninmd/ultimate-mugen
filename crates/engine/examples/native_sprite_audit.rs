use std::path::PathBuf;
use ume_engine::native::{audit_native_sprite_frames, NativeChar};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/chars");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let output = args.iter().find(|arg| !arg.starts_with("--")).ok_or("usage: native_sprite_audit <output.json> [--require-complete]")?;
    let strict = args.iter().any(|arg| arg == "--require-complete");
    let mut paths: Vec<_> = std::fs::read_dir(&root)?.filter_map(Result::ok).map(|entry| entry.path()).filter(|path| path.extension().is_some_and(|ext| ext == "json") && path.file_stem().is_some_and(|stem| stem != "index")).collect();
    paths.sort();
    let mut reports = Vec::new();
    let mut incomplete = Vec::new();
    for path in paths {
        let character: NativeChar = serde_json::from_slice(&std::fs::read(path)?)?;
        let mut report = audit_native_sprite_frames(&character);
        let animations = report["animations"].as_array().ok_or("missing animation report")?;
        let mut gaps = Vec::new();
        for animation in animations {
            let name = animation["animation"].as_str().unwrap_or_default();
            let unique = animation["uniqueBodyImages"].as_u64().unwrap_or(0);
            let minimum = match name { "idle" => 3, "walk" | "walkBack" => 6, "run" => 4, "powerUp" => 3, _ => 0 };
            if unique < minimum { gaps.push(format!("{name}: {unique} distinct body images, needs {minimum}")); }
            for frame in animation["frames"].as_array().ok_or("missing frame report")? {
                let resolution = frame["resolution"].as_str().unwrap_or("missing");
                if matches!(resolution, "idle-reuse" | "fallback-idle" | "missing") {
                    gaps.push(format!("{name}[{}] {}: {resolution}", frame["frame"], frame["pose"]));
                }
            }
        }
        if report["avatar"] != true { gaps.push("dedicated avatar missing".to_string()); }
        if !gaps.is_empty() { incomplete.push(character.id.clone()); }
        println!("{}: {} animation families; {} uncovered requirements", character.id, animations.len(), gaps.len());
        report["coverageComplete"] = serde_json::Value::Bool(gaps.is_empty());
        report["gaps"] = serde_json::to_value(gaps)?;
        reports.push(report);
    }
    std::fs::write(output, serde_json::to_vec_pretty(&reports)?)?;
    if strict && !incomplete.is_empty() {
        eprintln!("sprite coverage incomplete for: {}", incomplete.join(", "));
        std::process::exit(2);
    }
    Ok(())
}
