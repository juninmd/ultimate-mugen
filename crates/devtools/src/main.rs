use std::path::{Path, PathBuf};
use ume_devtools::{ability_reel, audit_moves, audit_sprites, motion, pack, story_npcs, thumbnails};

const USAGE: &str = "usage: ume-tools <command>\n\
  pack-sprites [out_root=web/dist] [--sprites dir]  pack assets/chars/sprites into web .pack files\n\
  stage-thumbs [stages=assets/stages]    write 240x135 thumbnails to <stages>/thumbs\n\
  audit-moves [dir=assets/chars] [--json] static roster audit (inputs, cancels, throws, hit boxes)\n\
  audit-sprites --out <report.json> [--root <repo>]  native PNG inventory\n\
  gen-story-npcs [chars=assets/chars]    regenerate assets/chars/npc_*.json (overwrites them)\n\
  ability-reel <frames_dir> <out_dir> [width=448] [colors=64] [fps=14] [--no-normals]  captioned GIF per fighter (needs ffmpeg)\n\
  motion-descriptor --character f --sprites dir --animation name --out file  sprite-gen asset descriptor\n\
  preview-motion|preview-scene --character f --sprites dir --animation name --sprite-gen exe --out dir [--loop|--cycles n]";

fn run(args: &[String]) -> Result<(), String> {
    let positional: Vec<&String> = args.iter().skip(1).filter(|a| !a.starts_with("--")).collect();
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    match args.first().map(String::as_str) {
        Some("pack-sprites") => {
            let out = PathBuf::from(positional.first().map(|s| s.as_str()).unwrap_or("web/dist"));
            let packed = pack::pack_all(Path::new(&flag("--sprites").unwrap_or_else(|| "assets/chars/sprites".into())), &out)?;
            let total: usize = packed.iter().map(|(_, size)| size).sum();
            for (id, size) in &packed {
                println!("  {id}.pack {:.2} MB", *size as f64 / 1048576.0);
            }
            println!("OK: {} ({:.1} MB)", out.join("assets/chars/sprites").display(), total as f64 / 1048576.0);
        }
        Some("stage-thumbs") => {
            let count = thumbnails::make_thumbnails(Path::new(positional.first().map(|s| s.as_str()).unwrap_or("assets/stages")))?;
            println!("OK: {count} thumbnails");
        }
        Some("audit-moves") => {
            let results = audit_moves::audit_directory(Path::new(positional.first().map(|s| s.as_str()).unwrap_or("assets/chars")))?;
            if args.iter().any(|a| a == "--json") {
                println!("{}", serde_json::to_string_pretty(&results).map_err(|e| e.to_string())?);
            } else {
                audit_moves::report_lines(&results).iter().for_each(|l| println!("{l}"));
            }
        }
        Some("audit-sprites") => {
            let out = flag("--out").ok_or("audit-sprites needs --out <report.json>")?;
            let report = audit_sprites::inspect(Path::new(&flag("--root").unwrap_or_else(|| ".".into())))?;
            std::fs::write(&out, serde_json::to_string_pretty(&report).map_err(|e| e.to_string())? + "\n").map_err(|e| format!("{out}: {e}"))?;
            audit_sprites::summary_lines(&report).iter().for_each(|l| println!("{l}"));
        }
        Some("gen-story-npcs") => {
            let written = story_npcs::generate(Path::new(positional.first().map(|s| s.as_str()).unwrap_or("assets/chars")))?;
            println!("OK: {}", written.join(", "));
        }
        Some("ability-reel") => {
            let num = |i: usize, default: usize| positional.get(i).map(|v| v.parse::<usize>().map_err(|e| e.to_string())).transpose().map(|v| v.unwrap_or(default));
            let options = ability_reel::ReelOptions { width: num(2, 448)?, colors: num(3, 64)? as u32, fps: num(4, 14)? as u32, skip_normals: args.iter().any(|a| a == "--no-normals") };
            let (Some(source), Some(out)) = (positional.first(), positional.get(1)) else { return Err(USAGE.to_string()) };
            ability_reel::make_reels(Path::new(source), Path::new(out), &options)?.iter().for_each(|l| println!("{l}"));
        }
        Some(cmd @ ("motion-descriptor" | "preview-motion" | "preview-scene")) => {
            let need = |name: &str| flag(name).ok_or(format!("{cmd} needs {name}"));
            let (character, sprites, animation, out) = (PathBuf::from(need("--character")?), PathBuf::from(need("--sprites")?), need("--animation")?, PathBuf::from(need("--out")?));
            if cmd == "motion-descriptor" {
                let (count, _) = motion::describe(&character, &sprites, &animation, &out)?;
                println!("Described {count} frames in {}", out.display());
            } else {
                let exe = PathBuf::from(need("--sprite-gen")?).canonicalize().map_err(|e| e.to_string())?;
                let output = if cmd == "preview-motion" {
                    motion::preview_motion(&character, &sprites, &animation, &out, &exe, args.iter().any(|a| a == "--loop"))?
                } else {
                    let cycles = flag("--cycles").map(|c| c.parse::<i64>().map_err(|e| e.to_string())).transpose()?.unwrap_or(1);
                    motion::preview_scene(&character, &sprites, &animation, &out, &exe, cycles)?
                };
                println!("Preview: {}; inspect motion and collision in the game", output.display());
            }
        }
        _ => return Err(USAGE.to_string()),
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
