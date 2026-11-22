use serde::Serialize;
use ume_engine::loader::MugenSources;
use ume_formats::ini::parse_ini_no_lines;

#[derive(Clone, Debug, Serialize)]
pub struct Route {
    pub command: String,
    pub input: String,
    pub state: i32,
    pub when: &'static str,
    pub cost: i32,
}

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

pub fn quoted_commands(text: &str) -> Vec<String> {
    let lower = text.to_ascii_lowercase();
    let b = lower.as_bytes();
    let mut out = vec![];
    let mut from = 0;
    while let Some(p) = lower[from..].find("command") {
        from += p + 7;
        let i = skip_ws(b, from);
        if b.get(i) != Some(&b'=') {
            continue;
        }
        let i = skip_ws(b, i + 1);
        if b.get(i) == Some(&b'"') {
            if let Some(end) = text[i + 1..].find('"') {
                out.push(text[i + 1..i + 1 + end].to_string());
            }
        }
    }
    out
}

fn number_after(text: &str, key: &str) -> Option<i32> {
    let pos = text.find(key)?;
    let digits: String = text[pos + key.len()..].trim_start().trim_start_matches(['=', '>']).trim_start().chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

pub fn routes(src: &MugenSources, commands: &[ume_engine::commands::CommandDef]) -> Vec<Route> {
    let mut out: Vec<Route> = vec![];
    let texts = std::iter::once(&src.cmd).chain(std::iter::once(&src.cns)).chain(src.st.iter()).chain(src.stcommon.iter());
    for t in texts {
        let mut in_minus1 = false;
        for s in parse_ini_no_lines(&t.text) {
            let name = s.lname.trim();
            if name.starts_with("statedef") {
                in_minus1 = name.trim_start_matches("statedef").trim() == "-1";
                continue;
            }
            if !in_minus1 || !s.get("type").is_some_and(|v| v.eq_ignore_ascii_case("changestate")) {
                continue;
            }
            let Some(state) = s.get("value").and_then(|v| v.trim().parse::<i32>().ok()) else { continue };
            let triggers: Vec<&str> = s.entries.iter().filter(|(k, _)| k.starts_with("trigger")).map(|(_, v)| v.as_str()).collect();
            let joined = triggers.join(" && ").to_ascii_lowercase().replace(' ', "");
            let Some(command) = quoted_commands(&triggers.join(" && ")).into_iter().find(|c| commands.iter().any(|d| d.name.eq_ignore_ascii_case(c) && !d.raw.is_empty())) else { continue };
            let when = if joined.contains("statetype=a") { "air" } else if joined.contains("statetype=c") { "crouch" } else { "stand" };
            let cost = number_after(&joined, "power>=").or_else(|| number_after(&joined, "power>")).unwrap_or(0);
            if out.iter().any(|r| r.command.eq_ignore_ascii_case(&command) && r.when == when) {
                continue;
            }
            let input = commands.iter().find(|d| d.name.eq_ignore_ascii_case(&command)).map(|d| d.raw.clone()).unwrap_or_default();
            out.push(Route { command, input, state, when, cost });
        }
    }
    out
}
