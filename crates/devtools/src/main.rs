use std::path::{Path, PathBuf};
use ume_devtools::{sff_cook, ability_reel, audit_moves, audit_sprites, debug_fight, homologate, knockdown_rate, fight_census, motion, mugen_census, pack, roster_audit, story_npcs, thumbnails};

const USAGE: &str = "usage: ume-tools <command>\n\
  pack-sprites [out_root=web/dist] [--sprites dir]  pack assets/chars/sprites into web .pack files\n\
  cook-sff <in.sff> <out.sff>             repack an .sff as LZ4 UMSF and verify it decodes to the same sprites\n\
  stage-thumbs [stages=assets/stages]    write 240x135 thumbnails to <stages>/thumbs\n\
  audit-moves [dir=assets/chars] [--json] static roster audit (inputs, cancels, throws, hit boxes)\n\
  audit-sprites --out <report.json> [--root <repo>]  native PNG inventory\n\
  debug-fight <char1> [char2] [--frames N] [--only-moves] [--json]  inspect frame data, triggers, clipboard and console trace\n\
  gen-story-npcs [chars=assets/chars]    regenerate assets/chars/npc_*.json (overwrites them)\n\
  mugen-census <zip-or-dir>...           count states, controllers, commands and actions of M.U.G.E.N packages\n\
  roster-audit [--chars dir] [--stages dir] [--out dir] [--limit N] [--fights N] [--only text] [--no-fights] [--fail-over severity]  severity report for every character and stage\n\
  homologate <dir=.> [--run] [--out report.md]  PASS/WARN/FAIL per character zip or folder; --run also loads the engine and ticks a fight\n\
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
        Some("cook-sff") => {
            let (Some(source), Some(target)) = (positional.first(), positional.get(1)) else { return Err(USAGE.to_string()) };
            let r = sff_cook::cook(Path::new(source.as_str()), Path::new(target.as_str()))?;
            println!("OK: {} sprites, {} -> {} bytes, decode {:.1} -> {:.1} ms", r.sprites, r.source_bytes, r.cooked_bytes, r.source_ms, r.cooked_ms);
        }
        Some("stage-thumbs") => {
            let count = thumbnails::make_thumbnails(Path::new(positional.first().map(|s| s.as_str()).unwrap_or("assets/stages")))?;
            println!("OK: {count} thumbnails");
        }
        Some("homologate") => {
            let root = PathBuf::from(positional.first().map(|s| s.as_str()).unwrap_or("."));
            let mut findings = Vec::new();
            for input in homologate::collect_inputs(&root) {
                match homologate::check_path(&input, args.iter().any(|a| a == "--run")) {
                    Ok(found) => findings.extend(found),
                    Err(error) => eprintln!("skip: {error}"),
                }
            }
            let markdown = homologate::report(&findings);
            match flag("--out") {
                Some(out) => std::fs::write(&out, &markdown).map_err(|e| e.to_string())?,
                None => print!("{markdown}"),
            }
        }
        Some("roster-audit") => {
            let number = |name: &str| flag(name).map(|v| v.parse::<usize>().map_err(|e| format!("{name}: {e}"))).transpose();
            let options = roster_audit::Options {
                chars_root: PathBuf::from(flag("--chars").unwrap_or_else(|| "mugen/chars".into())),
                stages_root: PathBuf::from(flag("--stages").unwrap_or_else(|| "mugen/stages".into())),
                out: PathBuf::from(flag("--out").unwrap_or_else(|| "docs/evidence/homologation".into())),
                chars: number("--limit")?,
                stages: number("--stage-limit")?,
                fights: number("--fights")?.unwrap_or(2),
                stage_fights: number("--stage-fights")?.unwrap_or(1),
                filter: flag("--only"),
                stage_filter: flag("--only-stage"),
                opponent: flag("--opponent"),
                skip_fights: args.iter().any(|a| a == "--no-fights"),
                quiet: args.iter().any(|a| a == "--quiet"),
                ..Default::default()
            };
            let report = roster_audit::run(&options)?;
            roster_audit::write_reports(&report, &options.out)?;
            roster_audit::console_lines(&report).iter().for_each(|line| println!("{line}"));
            println!("report: {}", options.out.display());
            if let Some(limit) = flag("--fail-over") {
                let limit = roster_audit::Severity::from_label(&limit).ok_or_else(|| format!("--fail-over: {limit} is not critical|major|minor|info"))?;
                if report.issues().any(|issue| issue.severity <= limit) {
                    std::process::exit(2);
                }
            }
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
        Some("debug-fight") => {
            let Some(p1) = positional.first() else { return Err(USAGE.to_string()) };
            let frames = flag("--frames").map(|v| v.parse::<usize>().map_err(|e| e.to_string())).transpose()?.unwrap_or(180);
            let opt = debug_fight::DebugOptions { p1: (*p1).clone(), p2: positional.get(1).map(|s| (*s).clone()), frames, only_moves: args.iter().any(|a| a == "--only-moves") };
            let val = debug_fight::run_debug(&opt)?;
            if args.iter().any(|a| a == "--json") {
                println!("{}", serde_json::to_string_pretty(&val).map_err(|e| e.to_string())?);
            } else {
                print!("{}", debug_fight::format_text(&val));
            }
        }
        Some("fight-census") => {
            let frames = flag("--frames").map(|v| v.parse::<usize>().map_err(|e| e.to_string())).transpose()?.unwrap_or(5400);
            let level = flag("--level").map(|v| v.parse::<i32>().map_err(|e| e.to_string())).transpose()?.unwrap_or(8);
            let roster: Vec<String> = flag("--roster").ok_or("fight-census needs --roster a,b,c")?.split(',').map(String::from).collect();
            let val = fight_census::run(&roster, level, flag("--level-b").map(|v| v.parse::<i32>().map_err(|e| e.to_string())).transpose()?.unwrap_or(level), frames)?;
            println!("{}", serde_json::to_string_pretty(&val).map_err(|e| e.to_string())?);
        }
        Some("knockdown-rate") => {
            let frames = flag("--frames").map(|v| v.parse::<usize>().map_err(|e| e.to_string())).transpose()?.unwrap_or(5400);
            let threshold = flag("--threshold").map(|v| v.parse::<f32>().map_err(|e| e.to_string())).transpose()?.unwrap_or(12.0);
            let rule = knockdown_rate::parse_rule(&flag("--rule").unwrap_or_else(|| "off".into()), threshold)?;
            let roster: Vec<String> = flag("--roster").ok_or("knockdown-rate needs --roster a,b,c")?.split(',').map(String::from).collect();
            let val = knockdown_rate::run(&roster, frames, rule)?;
            println!("{}", serde_json::to_string_pretty(&val).map_err(|e| e.to_string())?);
        }
        Some("gen-story-npcs") => {
            let written = story_npcs::generate(Path::new(positional.first().map(|s| s.as_str()).unwrap_or("assets/chars")))?;
            println!("OK: {}", written.join(", "));
        }
        Some("mugen-census") => {
            let all = positional.iter().map(|path| mugen_census::census_path(Path::new(path.as_str()))).collect::<Result<Vec<_>, _>>()?;
            mugen_census::report(&all).iter().for_each(|l| println!("{l}"));
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
