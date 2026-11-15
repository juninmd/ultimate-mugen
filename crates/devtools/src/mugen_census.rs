use std::collections::BTreeMap;
use std::path::Path;
use ume_formats::vfs::Vfs;

const CODE_EXTENSIONS: [&str; 4] = [".cns", ".st", ".cmd", ".air"];
const TOP_CONTROLLERS: usize = 8;
const COMMON_STATE_LIMIT: i64 = 200;
const HIT_STATES: std::ops::Range<i64> = 5000..6000;

const UMF_EQUIVALENTS: [(&str, &str); 12] = [
    (
        "changestate",
        "generated from `cancels`, `when` and move `kind`",
    ),
    (
        "varset",
        "engine systems (`cost`, `requires`, passives, forms)",
    ),
    (
        "varadd",
        "engine systems (`cost`, `requires`, passives, forms)",
    ),
    ("explod", "`fx`, `spark`, `hitAnim`"),
    ("playsnd", "`voice` and per-level hit sounds"),
    ("hitdef", "`hits[]`"),
    ("velset", "`moves[]` velocity changes"),
    ("veladd", "`moves[]` velocity changes"),
    ("destroyself", "projectile `removeTime`"),
    ("helper", "`projectile`, `throw`, abilities"),
    ("changeanim", "`anim` per move"),
    ("assertspecial", "engine flags"),
];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Census {
    pub name: String,
    pub files: usize,
    pub lines: usize,
    pub statedefs: usize,
    pub common_statedefs: usize,
    pub controllers: BTreeMap<String, usize>,
    pub commands: usize,
    pub ai_commands: usize,
    pub actions: usize,
}

impl Census {
    pub fn controller_total(&self) -> usize {
        self.controllers.values().sum()
    }

    pub fn top_controllers(&self) -> Vec<(&str, usize)> {
        let mut ranked: Vec<(&str, usize)> = self
            .controllers
            .iter()
            .map(|(name, count)| (name.as_str(), *count))
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        ranked.truncate(TOP_CONTROLLERS);
        ranked
    }
}

#[derive(PartialEq)]
enum Section {
    Other,
    Statedef,
    State,
    Command,
}

fn value_of<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let (name, value) = line.split_once('=')?;
    name.trim().eq_ignore_ascii_case(key).then(|| value.trim())
}

pub fn scan_text(census: &mut Census, text: &str) {
    let mut section = Section::Other;
    for raw in text.lines() {
        let line = raw.split(';').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        census.lines += 1;
        if let Some(header) = line
            .strip_prefix('[')
            .and_then(|rest| rest.split(']').next())
        {
            let lower = header.trim().to_ascii_lowercase();
            section = if let Some(id) = lower.strip_prefix("statedef") {
                census.statedefs += 1;
                let id = id
                    .trim()
                    .split([',', ' '])
                    .next()
                    .and_then(|id| id.parse::<i64>().ok())
                    .unwrap_or(0);
                if id < COMMON_STATE_LIMIT || HIT_STATES.contains(&id) {
                    census.common_statedefs += 1;
                }
                Section::Statedef
            } else if lower.starts_with("state ") {
                Section::State
            } else if lower == "command" {
                census.commands += 1;
                Section::Command
            } else {
                if lower.starts_with("begin action") {
                    census.actions += 1;
                }
                Section::Other
            };
            continue;
        }
        match section {
            Section::State => {
                if let Some(kind) = value_of(line, "type") {
                    *census
                        .controllers
                        .entry(kind.to_ascii_lowercase())
                        .or_default() += 1;
                }
            }
            Section::Command => {
                if value_of(line, "name").is_some_and(|name| {
                    name.trim_matches('"')
                        .to_ascii_lowercase()
                        .starts_with("ai")
                }) {
                    census.ai_commands += 1;
                }
            }
            _ => {}
        }
    }
}

pub fn census_of(name: &str, vfs: &Vfs) -> Census {
    let mut census = Census {
        name: name.to_string(),
        ..Census::default()
    };
    for (key, file) in vfs.iter() {
        let lower = key.to_ascii_lowercase();
        if CODE_EXTENSIONS
            .iter()
            .any(|extension| lower.ends_with(extension))
        {
            census.files += 1;
            scan_text(&mut census, &String::from_utf8_lossy(file.as_slice()));
        }
    }
    census
}

pub fn census_path(path: &Path) -> Result<Census, String> {
    let mut vfs = Vfs::new();
    let name = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    if path.is_dir() {
        ume_formats::vfs::add_dir(&mut vfs, path, "")
            .map_err(|error| format!("{}: {error}", path.display()))?;
    } else {
        let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        ume_formats::vfs::add_zip(&mut vfs, &bytes, "")
            .map_err(|error| format!("{}: {error:?}", path.display()))?;
    }
    Ok(census_of(&name, &vfs))
}

pub fn report(all: &[Census]) -> Vec<String> {
    let mut lines = vec![
        "| Package | Code files | Code lines | Statedefs | Common states redeclared | Controllers | Commands | AI commands | AIR actions |".to_string(),
        "|---|---|---|---|---|---|---|---|---|".to_string(),
    ];
    let mut totals: BTreeMap<String, usize> = BTreeMap::new();
    for census in all {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            census.name,
            census.files,
            census.lines,
            census.statedefs,
            census.common_statedefs,
            census.controller_total(),
            census.commands,
            census.ai_commands,
            census.actions
        ));
        for (name, count) in &census.controllers {
            *totals.entry(name.clone()).or_default() += count;
        }
    }
    let merged = Census {
        controllers: totals,
        ..Census::default()
    };
    let total = merged.controller_total().max(1);
    lines.push(String::new());
    lines.push("| Controller | Uses | Share | UMF equivalent |".to_string());
    lines.push("|---|---|---|---|".to_string());
    for (name, count) in merged.top_controllers() {
        let equivalent = UMF_EQUIVALENTS
            .iter()
            .find(|(key, _)| *key == name)
            .map_or("hand-written state", |(_, text)| text);
        lines.push(format!(
            "| `{name}` | {count} | {:.0}% | {equivalent} |",
            count as f64 * 100.0 / total as f64
        ));
    }
    lines
}

#[cfg(test)]
#[path = "mugen_census_tests.rs"]
mod tests;
