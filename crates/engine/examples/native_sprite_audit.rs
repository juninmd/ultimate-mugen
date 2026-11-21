use std::path::PathBuf;
use ume_native::{audit_native_sprite_frames, NativeChar};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/chars");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let output = args.iter().find(|arg| !arg.starts_with("--")).ok_or("usage: native_sprite_audit <output.json> [--require-complete] [--require-hit-reactions]")?;
    let strict = args.iter().any(|arg| arg == "--require-complete");
    let strict_reactions = args.iter().any(|arg| arg == "--require-hit-reactions");
    let mut paths: Vec<_> = std::fs::read_dir(&root)?.filter_map(Result::ok).map(|entry| entry.path()).filter(|path| path.extension().is_some_and(|ext| ext == "json") && path.file_stem().is_some_and(|stem| stem != "index")).collect();
    paths.sort();
    let mut reports = Vec::new();
    let mut incomplete = Vec::new();
    let mut incomplete_reactions = Vec::new();
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
        let reaction_gaps = hit_reaction_gaps(animations);
        if !reaction_gaps.is_empty() {
            incomplete_reactions.push(character.id.clone());
        }
        println!(
            "{}: {} animation families, {} sprite gaps, {} hit reaction gaps",
            character.id, animations.len(), gaps.len(), reaction_gaps.len()
        );
        report["hitReactionGaps"] = serde_json::to_value(reaction_gaps)?;
        report["coverageComplete"] = serde_json::Value::Bool(gaps.is_empty());
        report["gaps"] = serde_json::to_value(gaps)?;
        reports.push(report);
    }
    std::fs::write(output, serde_json::to_vec_pretty(&reports)?)?;
    let sprite_failure = strict && !incomplete.is_empty();
    let reaction_failure = strict_reactions && !incomplete_reactions.is_empty();
    if sprite_failure {
        eprintln!("sprite coverage incomplete for: {}", incomplete.join(", "));
    }
    if reaction_failure {
        eprintln!("hit reaction coverage incomplete for: {}", incomplete_reactions.join(", "));
    }
    if sprite_failure || reaction_failure {
        std::process::exit(2);
    }
    Ok(())
}

fn hit_reaction_gaps(animations: &[serde_json::Value]) -> Vec<&'static str> {
    let frames = |name: &str| {
        animations
            .iter()
            .find(|item| item["animation"] == name)
            .and_then(|item| item["frames"].as_array())
            .map(Vec::as_slice)
            .unwrap_or_default()
    };
    let fall = frames("fall");
    [
        "hitLight",
        "hitMedium",
        "hitHeavy",
        "hitLowLight",
        "hitLowMedium",
        "hitLowHeavy",
        "hitCrouch",
        "hitAir",
    ]
    .into_iter()
    .filter(|&name| {
        let [impact, recoil, ..] = frames(name) else {
            return true;
        };
        let first = impact["pixelFingerprint"].as_str();
        let second = recoil["pixelFingerprint"].as_str();
        first.is_none()
            || second.is_none()
            || first == second
            || name == "hitAir"
                && [impact, recoil].into_iter().any(|frame| {
                    frame["resolution"] != "exact"
                        || fall.iter().any(|fall_frame| {
                            frame["pixelFingerprint"] == fall_frame["pixelFingerprint"]
                        })
                })
    })
    .collect()
}
