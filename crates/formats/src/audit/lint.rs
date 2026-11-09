use crate::ini::{parse_ini_no_lines, Section};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub file: String,
    pub section: String,
    pub key: String,
    pub message: String,
}

const NOHITPAUSE_TYPES: &[&str] = &[
    "angledraw",
    "playerpush",
    "offset",
    "screenbound",
    "trans",
    "width",
];

pub fn lint_text(file: &str, text: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for section in parse_ini_no_lines(text) {
        lint_section(file, &section, &mut out);
    }
    out
}

fn push(out: &mut Vec<Finding>, file: &str, section: &Section, key: &str, message: String) {
    out.push(Finding {
        file: file.to_string(),
        section: section.name.clone(),
        key: key.to_string(),
        message,
    });
}

fn numbers(value: &str) -> Option<Vec<i32>> {
    value
        .split(',')
        .map(|part| part.trim().parse::<i32>().ok())
        .collect()
}

fn join(values: &[i32]) -> String {
    values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn lint_section(file: &str, section: &Section, out: &mut Vec<Finding>) {
    let kind = section.get("type").map(|t| t.trim().to_ascii_lowercase());
    if let Some(volume) = section.get("volume") {
        match volume.trim().parse::<i32>() {
            Ok(v) => {
                let scaled = ((v + 255) as f32 / 512.0 * 100.0).round() as i32;
                push(
                    out,
                    file,
                    section,
                    "volume",
                    format!("legacy volume {v}; use volumescale = {scaled}"),
                );
            }
            Err(_) => push(
                out,
                file,
                section,
                "volume",
                "legacy volume is not an integer".into(),
            ),
        }
    }
    if section.lname.starts_with("statedef") {
        if let Some(velset) = section.get("velset").and_then(numbers) {
            if velset.len() > 2 {
                push(
                    out,
                    file,
                    section,
                    "velset",
                    format!(
                        "velset has a z component; use velset = {}",
                        join(&velset[..2])
                    ),
                );
            }
        }
    }
    if kind.as_deref() == Some("hitdef") {
        lint_hitdef(file, section, out);
    }
    if let Some(kind) = kind.as_deref() {
        if NOHITPAUSE_TYPES.contains(&kind) && section.get("ignorehitpause").is_none() {
            push(
                out,
                file,
                section,
                "ignorehitpause",
                "controller ignores hitpause in M.U.G.E.N; add ignorehitpause = 1".into(),
            );
        }
    }
}

fn lint_hitdef(file: &str, section: &Section, out: &mut Vec<Finding>) {
    for key in ["snap", "mindist", "maxdist"] {
        if let Some(values) = section.get(key).and_then(numbers) {
            if values.len() > 2 {
                push(
                    out,
                    file,
                    section,
                    key,
                    format!(
                        "{key} has a third index; use {key} = {}",
                        join(&values[..2])
                    ),
                );
            }
        }
    }
}
