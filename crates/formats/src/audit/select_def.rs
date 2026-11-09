const EMPTY_SLOT: &str = "-";
const OVERFLOW_NOTE: &str = ";below are characters that do not fit in the template slots";

fn group_of(path: &str) -> &str {
    path.split_once('/').map_or("", |(group, _)| group)
}

fn sorted_unique(paths: &[String]) -> Vec<String> {
    let mut out: Vec<String> = paths.iter().map(|p| p.replace('\\', "/")).collect();
    out.sort_by(|a, b| group_of(a).cmp(group_of(b)).then_with(|| a.cmp(b)));
    out.dedup();
    out
}

fn character_line(path: &str) -> String {
    format!("\"{path}\", random, order=1")
}

fn stages_block(stages: &[String]) -> Vec<String> {
    let mut lines = vec!["[ExtraStages]".to_string()];
    lines.extend(
        stages
            .iter()
            .map(|s| format!("\"{}\"", s.replace('\\', "/"))),
    );
    lines
}

pub fn build_select(characters: &[String], stages: &[String], template: Option<&str>) -> String {
    let characters = sorted_unique(characters);
    let stages = sorted_unique(stages);
    let lines = match template {
        Some(template) => from_template(template, &characters, &stages),
        None => grouped(&characters, &stages),
    };
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn grouped(characters: &[String], stages: &[String]) -> Vec<String> {
    let mut lines = vec!["[Characters]".to_string()];
    let mut current: Option<&str> = None;
    for path in characters {
        let group = group_of(path);
        if current != Some(group) {
            if !group.is_empty() {
                lines.push(format!(";{group}"));
            }
            current = Some(group);
        }
        lines.push(character_line(path));
    }
    lines.extend(stages_block(stages));
    lines
}

fn from_template(template: &str, characters: &[String], stages: &[String]) -> Vec<String> {
    let mut pending = characters.iter();
    let mut lines = Vec::new();
    let mut has_stages = false;
    for line in template.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("randomselect") {
            lines.push(match pending.next() {
                Some(path) => character_line(path),
                None => EMPTY_SLOT.to_string(),
            });
        } else {
            has_stages |= lower == "[extrastages]";
            lines.push(line.to_string());
        }
    }
    let overflow: Vec<&String> = pending.collect();
    if !overflow.is_empty() {
        lines.push(OVERFLOW_NOTE.to_string());
        lines.extend(overflow.iter().map(|p| character_line(p)));
    }
    if !has_stages && !stages.is_empty() {
        lines.extend(stages_block(stages));
    }
    lines
}
