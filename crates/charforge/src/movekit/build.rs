use super::animation::build_action;
use super::commands::{command_entries, entry_block};
use super::kinds::KINDS;
use super::phases::expand_phases;
use super::sprites::SpriteLibrary;
use super::statedef::build_state;
use super::value::{get_int, get_text, tables};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use toml::Table;

pub struct Built {
    pub air: String,
    pub cns: String,
    pub cmd: String,
    pub entry: String,
}

pub fn validate_moves(moves: &[Table]) -> Result<(), String> {
    let mut seen = HashSet::new();
    for move_ in moves {
        let state = move_.get("state").and_then(|v| v.as_integer()).ok_or("every move needs an integer state")?;
        let kind = get_text(move_, "kind").unwrap_or_else(|| "normal".into());
        if !KINDS.contains(&kind.as_str()) {
            return Err(format!("state {state}: kind must be one of {KINDS:?}"));
        }
        for expanded in expand_phases(move_)? {
            if !seen.insert(get_int(&expanded, "state", 0)) {
                return Err(format!("state {} is defined twice", get_int(&expanded, "state", 0)));
            }
        }
    }
    Ok(())
}

pub fn build_moves(moves: &[Table], defaults: &Table, library: &mut SpriteLibrary) -> Result<Built, String> {
    validate_moves(moves)?;
    let height = defaults.get("clsn1_height").and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)));
    let (mut air, mut cns) = (Vec::new(), String::new());
    for move_ in moves {
        for state in expand_phases(move_)? {
            let mut text = String::new();
            if state.contains_key("frames") {
                text = build_action(&state, library, height)?.1;
                air.push(text.clone());
            }
            cns += &build_state(&state, defaults, &text)?;
        }
    }
    Ok(Built { air: air.join("\n"), cns, cmd: command_entries(moves), entry: entry_block(moves) })
}

pub fn build_from_spec(spec_path: &Path, out_dir: &Path) -> Result<Built, String> {
    let text = std::fs::read_to_string(spec_path).map_err(|e| format!("{}: {e}", spec_path.display()))?;
    let spec: Table = text.parse().map_err(|e| format!("{}: {e}", spec_path.display()))?;
    let defaults = spec.get("defaults").and_then(|v| v.as_table()).cloned().unwrap_or_default();
    let base = spec_path.parent().unwrap_or(Path::new("."));
    let manifests: Vec<PathBuf> = super::value::list(&defaults, "sprite_manifests").into_iter().filter_map(|v| v.as_str()).map(|p| base.join(p)).collect();
    let mut library = SpriteLibrary::from_manifests(&manifests)?;
    let built = build_moves(&tables(&spec, "move").into_iter().cloned().collect::<Vec<_>>(), &defaults, &mut library)?;
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    for (name, content) in [("moves.air", &built.air), ("moves.cns", &built.cns), ("moves.cmd", &built.cmd), ("statedef-1.cns", &built.entry)] {
        std::fs::write(out_dir.join(name), content).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(built)
}
