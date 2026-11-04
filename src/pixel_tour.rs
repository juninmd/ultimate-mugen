use crate::app::App;
use crate::shots::fight;

const ATTACKER_SHOTS: [(&str, usize, &str); 3] = [("idle", 120, "faceoff"), ("transform-start", 100, "transform"), ("transform-done", 100, "awake")];
const VICTIM_SHOTS: [(&str, usize, &str); 3] = [("hit", 100, "hit"), ("knockout", 100, "ko"), ("defeat", 100, "defeat")];

pub async fn run(app: &mut App, dir: &str) {
    let chars: Vec<String> = std::env::var("UME_CHARS").unwrap_or_default().split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
    if chars.len() < 2 { println!("set UME_CHARS=pixel_art_char,opponent"); return; }
    let stage = std::env::var("UME_STAGE").unwrap_or_else(|_| "dojo".into());
    let (hero, rival) = (&chars[0], &chars[1]);
    for (name, ticks, want) in ATTACKER_SHOTS { fight(app, dir, &format!("{hero}-{name}"), hero, rival, &stage, ticks, want).await; }
    for (name, ticks, want) in VICTIM_SHOTS { fight(app, dir, &format!("{hero}-{name}"), rival, hero, &stage, ticks, want).await; }
}
