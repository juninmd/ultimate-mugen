use super::{Issue, Severity};
use std::collections::BTreeSet;
use ume_engine::chardef::{CharDef, Controller};
use ume_engine::diagnostics::Category;

const TRANSPARENT_FRAME_SAMPLES: usize = 4;
const MISSING_SPRITE_SAMPLES: usize = 5;

pub fn static_issues(d: &CharDef) -> Vec<Issue> {
    let mut issues = Vec::new();
    let subject = d.name.clone();
    if d.anims.is_empty() {
        issues.push(Issue::new(Severity::Critical, "char", "char-no-anims", &subject, "the AIR file produced no animation"));
        return issues;
    }
    if d.states.is_empty() {
        issues.push(Issue::new(Severity::Critical, "char", "char-no-states", &subject, "no state could be parsed from the CNS files"));
    }
    if d.commands.is_empty() {
        issues.push(Issue::new(Severity::Critical, "char", "char-no-commands", &subject, "the CMD file produced no command, the CPU cannot attack"));
    }
    diagnostics(d, &subject, &mut issues);
    sprites(d, &subject, &mut issues);
    sounds(d, &subject, &mut issues);
    issues
}

fn diagnostics(d: &CharDef, subject: &str, issues: &mut Vec<Issue>) {
    let diag = d.diagnostics();
    for (category, severity, code) in [
        (Category::Controller, Severity::Major, "char-unsupported-controller"),
        (Category::Trigger, Severity::Major, "char-unknown-trigger"),
        (Category::Redirection, Severity::Major, "char-unknown-redirection"),
        (Category::StringTrigger, Severity::Minor, "char-unknown-string-trigger"),
        (Category::Parameter, Severity::Minor, "char-unknown-parameter"),
        (Category::AssertFlag, Severity::Minor, "char-unknown-assert-flag"),
    ] {
        for (name, hits) in diag.distinct(category) {
            let place = hits.first().map(|h| format!("{} state {}", h.file, h.state)).unwrap_or_default();
            let hint = if category == Category::Trigger { " (unknown trigger, or a sound/sprite prefix the expression parser could not evaluate)" } else { "" };
            issues.push(Issue::new(severity, "char", code, subject, format!("{name} ({} use(s), first at {place}){hint}", hits.len())));
        }
    }
}

fn sprites(d: &CharDef, subject: &str, issues: &mut Vec<Issue>) {
    let mut missing: BTreeSet<(i32, i32, i32)> = BTreeSet::new();
    let mut blank: BTreeSet<(i32, i32, i32)> = BTreeSet::new();
    for (no, anim) in &d.anims {
        for frame in &anim.frames {
            let key = (frame.group, frame.index);
            match d.sprites.get(&key) {
                None => {
                    missing.insert((*no, frame.group, frame.index));
                }
                Some(sprite) => {
                    if sprite.width <= 0.0 || sprite.height <= 0.0 {
                        missing.insert((*no, frame.group, frame.index));
                        continue;
                    }
                    let empty = match d.indexed.get(&key) {
                        Some(indexed) => indexed.pixels.iter().all(|p| *p == 0),
                        None => sprite.image.as_ref().is_none_or(|image| image.rgba().chunks(4).all(|p| p[3] == 0)),
                    };
                    if empty {
                        blank.insert((*no, frame.group, frame.index));
                    }
                }
            }
        }
    }
    let anims: BTreeSet<i32> = missing.iter().map(|m| m.0).collect();
    for (no, group, index) in missing.iter().take(MISSING_SPRITE_SAMPLES) {
        issues.push(Issue::new(Severity::Major, "sprite", "char-sprite-missing", subject, format!("anim {no} uses sprite {group},{index} that the .sff does not contain")));
    }
    if missing.len() > MISSING_SPRITE_SAMPLES {
        issues.push(Issue::new(Severity::Major, "sprite", "char-sprite-missing", subject, format!("{} more missing sprite reference(s) across {} animation(s)", missing.len() - MISSING_SPRITE_SAMPLES, anims.len())));
    }
    for (no, group, index) in blank.iter().take(TRANSPARENT_FRAME_SAMPLES) {
        issues.push(Issue::new(Severity::Minor, "sprite", "char-sprite-transparent", subject, format!("anim {no} draws sprite {group},{index} with every pixel transparent (invisible frame)")));
    }
    if blank.len() > TRANSPARENT_FRAME_SAMPLES {
        issues.push(Issue::new(Severity::Minor, "sprite", "char-sprite-transparent", subject, format!("{} more fully transparent frame(s)", blank.len() - TRANSPARENT_FRAME_SAMPLES)));
    }
}

fn sounds(d: &CharDef, subject: &str, issues: &mut Vec<Issue>) {
    let mut missing: BTreeSet<(i32, i32)> = BTreeSet::new();
    let mut total = 0usize;
    for state in d.states.values() {
        for controller in &state.controllers {
            if !is_playsnd(controller) {
                continue;
            }
            let Some(value) = controller.params.iter().find(|(k, _)| k == "value").map(|(_, p)| p.raw.trim()) else { continue };
            total += 1;
            let Some(key) = literal_sound(value) else { continue };
            if !d.sounds.contains_key(&key) {
                missing.insert(key);
            }
        }
    }
    for (group, index) in missing.iter().take(6) {
        issues.push(Issue::new(Severity::Major, "sound", "char-sound-missing", subject, format!("playsnd {group},{index} has no sample in the .snd ({total} literal sound call(s) audited)")));
    }
    if missing.len() > 6 {
        issues.push(Issue::new(Severity::Major, "sound", "char-sound-missing", subject, format!("{} more missing sound reference(s)", missing.len() - 6)));
    }
    if total == 0 && d.sounds.is_empty() {
        issues.push(Issue::new(Severity::Info, "sound", "char-silent", subject, "the character has no sound at all"));
    }
}

fn is_playsnd(controller: &Controller) -> bool {
    matches!(controller.kind.as_str(), "playsnd" | "playse")
}

fn literal_sound(raw: &str) -> Option<(i32, i32)> {
    let mut parts = raw.split(',');
    let group = parts.next()?.trim().parse::<i32>().ok()?;
    let index = parts.next().unwrap_or("0").trim().parse::<i32>().ok()?;
    Some((group, index))
}
