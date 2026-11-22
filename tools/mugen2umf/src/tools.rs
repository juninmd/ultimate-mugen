use crate::{bundle, convert, extra, inspect, parity, profile, routes, source};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

pub type Args = BTreeMap<String, String>;

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub params: &'static [(&'static str, &'static str, bool)],
    pub run: fn(&Args) -> Result<String, String>,
}

fn arg<'a>(a: &'a Args, k: &str) -> Result<&'a str, String> {
    a.get(k).map(String::as_str).ok_or_else(|| format!("missing argument: {k}"))
}

fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_default()
}

fn run_parity(src: &source::Source, out: &Path) -> Result<parity::Parity, String> {
    let sources = ume_engine::loader::read_mugen_sources(&src.vfs, &src.def)?;
    let orig = Arc::new(ume_engine::loader::assemble_mugen_char(sources.clone(), 1, true));
    let loaded = bundle::load(out)?;
    let umf = Arc::new(bundle::compile(&loaded, 1, true)?);
    let routes = routes::routes(&sources, &orig.commands);
    let report = parity::parity(&loaded.fighter.name, &orig, &umf, &routes);
    let dir = loaded.json_path.parent().unwrap_or(Path::new("."));
    std::fs::write(dir.join("parity.md"), report.markdown()).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("routes.json"), serde_json::to_string_pretty(&routes).unwrap_or_default()).map_err(|e| e.to_string())?;
    Ok(report)
}

fn convert_tool(a: &Args) -> Result<String, String> {
    let src = source::open(Path::new(arg(a, "src")?))?;
    let (out, id) = (Path::new(arg(a, "out")?), arg(a, "id")?);
    let mut b = convert::convert(&src, id, a.get("name").map(String::as_str))?;
    let mut applied = vec![];
    if let Some(path) = a.get("extra") {
        let raw = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        applied = extra::apply(&mut b.json, &serde_json::from_str(&raw).map_err(|e| format!("{path}: {e}"))?)?;
    }
    convert::write(&b, out, id)?;
    ume_native::parse_char(&std::fs::read_to_string(out.join(format!("{id}.json"))).map_err(|e| e.to_string())?)?;
    let info = inspect::inspect(&src)?;
    std::fs::write(out.join("report.md"), format!("# {id}\n\nSource: `{}`\n\n```json\n{}\n```\n", src.def, pretty(&info))).map_err(|e| e.to_string())?;
    let report = run_parity(&src, out)?;
    Ok(format!("wrote {} ({} files, extras: {})\n{}", out.display(), b.files.len() + 1, if applied.is_empty() { "none".into() } else { applied.join(", ") }, report.summary()))
}

fn parity_tool(a: &Args) -> Result<String, String> {
    let src = source::open(Path::new(arg(a, "src")?))?;
    let report = run_parity(&src, Path::new(arg(a, "out")?))?;
    Ok(if a.get("full").is_some() { report.markdown() } else { report.summary() })
}

fn evidence_tool(a: &Args) -> Result<String, String> {
    let src = source::open(Path::new(arg(a, "src")?))?;
    let out = Path::new(arg(a, "out")?);
    let report = run_parity(&src, out)?;
    let wanted: Vec<String> = a.get("moves").map(|m| m.split(',').map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()).collect()).unwrap_or_default();
    let mut picked: Vec<&parity::MoveParity> = report.moves.iter().filter(|m| m.original.states.contains(&m.route.state)).collect();
    if wanted.is_empty() {
        picked.sort_by(|x, y| y.original.damage.total_cmp(&x.original.damage));
        picked.truncate(6);
    } else {
        picked.retain(|m| wanted.contains(&m.route.command.to_ascii_lowercase()));
    }
    let routes: Vec<routes::Route> = picked.iter().map(|m| m.route.clone()).collect();
    let sources = ume_engine::loader::read_mugen_sources(&src.vfs, &src.def)?;
    let orig = Arc::new(ume_engine::loader::assemble_mugen_char(sources, 1, true));
    let umf = Arc::new(bundle::compile(&bundle::load(out)?, 1, true)?);
    let dir = bundle::find_json(out)?.with_file_name("evidence");
    let written = parity::evidence(&orig, &umf, &routes, &dir)?;
    Ok(format!("{}\n{} sheets (top row original, bottom row UMF):\n{}", report.summary(), written.len(), written.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n")))
}

fn validate_tool(a: &Args) -> Result<String, String> {
    let loaded = bundle::load(Path::new(arg(a, "out")?))?;
    let d = bundle::compile(&loaded, 1, true)?;
    Ok(format!("{}: valid UMF ({} states, {} animations, {} sprites, {} sounds, {} commands)", loaded.json_path.display(), d.states.len(), d.anims.len(), d.sprites.len(), d.sounds.len(), d.commands.len()))
}

fn set_profile_tool(a: &Args) -> Result<String, String> {
    let json_path = bundle::find_json(Path::new(arg(a, "out")?))?;
    let patch: Value = serde_json::from_str(arg(a, "profile")?).map_err(|e| format!("profile: {e}"))?;
    profile::apply(&json_path, &patch)
}

fn read_report(a: &Args) -> Result<String, String> {
    let json_path = bundle::find_json(Path::new(arg(a, "out")?))?;
    std::fs::read_to_string(json_path.with_file_name("parity.md")).map_err(|e| format!("no parity.md yet: {e}"))
}

pub const TOOLS: &[Tool] = &[
    Tool { name: "convert", description: "Migrate a M.U.G.E.N character (folder or .zip) to a UMF bundle with identical behavior, then write report.md, routes.json and the move-by-move parity.md.", params: &[("src", "M.U.G.E.N character folder or .zip", true), ("out", "output bundle folder", true), ("id", "fighter id", true), ("name", "display name (default: the .def displayname)", false), ("extra", "JSON file merged on top: profile, quotes, scripts, mugenRoutes, UMF moves", false)], run: convert_tool },
    Tool { name: "inspect", description: "Summarize a M.U.G.E.N character: files, SFF, sounds, animations, commands, states, controllers, command routes and engine support diagnostics.", params: &[("src", "M.U.G.E.N character folder or .zip", true)], run: |a| Ok(pretty(&inspect::inspect(&source::open(Path::new(arg(a, "src")?))?)?)) },
    Tool { name: "hitdefs", description: "List every HitDef of a M.U.G.E.N character with its state and main values.", params: &[("src", "M.U.G.E.N character folder or .zip", true)], run: |a| Ok(pretty(&inspect::hitdefs(&source::open(Path::new(arg(a, "src")?))?)?)) },
    Tool { name: "parity", description: "Play every command of the original and of the UMF bundle tick by tick and compare data and behavior; writes parity.md.", params: &[("src", "original M.U.G.E.N character folder or .zip", true), ("out", "UMF bundle folder", true), ("full", "any value: return the whole markdown report", false)], run: parity_tool },
    Tool { name: "evidence", description: "Render side-by-side sheets of moves (top row original M.U.G.E.N, bottom row UMF) into <out>/evidence/.", params: &[("src", "original M.U.G.E.N character folder or .zip", true), ("out", "UMF bundle folder", true), ("moves", "comma-separated command names (default: the 6 that deal the most damage)", false)], run: evidence_tool },
    Tool { name: "validate", description: "Load and compile a UMF bundle exactly as the game does.", params: &[("out", "UMF bundle folder or .json", true)], run: validate_tool },
    Tool { name: "report", description: "Read the parity.md of a converted bundle.", params: &[("out", "UMF bundle folder", true)], run: read_report },
    Tool { name: "profile_facts", description: "Gather what the package says about the character (.def [Info], readme and notes) plus the UMF-only fields to fill (country, nationality, origin, bio, title, fighting style, series, quotes).", params: &[("src", "M.U.G.E.N character folder or .zip", true)], run: |a| Ok(pretty(&profile::facts(&source::open(Path::new(arg(a, "src")?))?))) },
    Tool { name: "set_profile", description: "Write UMF-only fields into a bundle (profile in en/pt/es, series, description, quotes) after validating them.", params: &[("out", "UMF bundle folder or .json", true), ("profile", "JSON object, e.g. {\"series\":\"Marvel\",\"profile\":{\"country\":\"RU\"}}", true)], run: set_profile_tool },
];
