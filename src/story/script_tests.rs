use super::*;
use crate::story::STAGES;

#[test]
fn embedded_default_toml_parses_and_matches_builtin_stages() {
    let script = StoryScript::default_campaign();
    assert!(script.campaign.enabled);
    assert_eq!(script.stages.len(), STAGES.len());
    for (i, (parsed, builtin)) in script.stages.iter().zip(STAGES.iter()).enumerate() {
        assert_eq!(parsed.arena, builtin.arena, "stage {i} arena");
        assert_eq!(parsed.boss, builtin.boss, "stage {i} boss");
        assert_eq!(parsed.alt_boss, builtin.alt_boss, "stage {i} alt_boss");
        assert_eq!(parsed.waves.len(), builtin.waves.len(), "stage {i} waves");
        assert!(!script.stage_title(i, Lang::En).is_empty());
        assert!(!script.stage_title(i, Lang::Pt).is_empty());
        assert!(!script.stage_title(i, Lang::Es).is_empty());
        assert!(!script.stage_beat(i, 0, Lang::En).is_empty());
        assert!(!script.stage_beat(i, 3, Lang::Pt).is_empty());
    }
}

#[test]
fn custom_toml_supports_plain_strings_and_custom_rules() {
    let raw = r#"
[campaign]
id = "cyber_rebellion"
title = "CYBER REBELLION"
enabled = true
credits = 5
wave_recovery = 0.35
stage_recovery = 0.8
boss_life_factor = 2.0
boss_attack_factor = 1.25

[[stages]]
arena = "neon"
title = "SECTOR ZERO"
boss = "volta"
alt_boss = "akio"
waves = [["npc_razor"], ["npc_phantom", "npc_spitfire"]]
beats = ["The grid is locked.", "Break the firewall."]

[[stages]]
arena = "dojo"
title = "CORE TOWER"
boss = "oboro"
waves = [["npc_colossus"]]
beats = ["Final boss awaits."]
"#;
    let script = StoryScript::parse(raw).expect("valid custom script");
    assert_eq!(script.campaign.id, "cyber_rebellion");
    assert_eq!(script.title(Lang::En), "CYBER REBELLION");
    assert_eq!(script.title(Lang::Pt), "CYBER REBELLION");
    assert_eq!(script.campaign.credits, 5);
    assert!((script.campaign.boss_life_factor - 2.0).abs() < 1e-6);
    assert_eq!(script.stages.len(), 2);
    assert_eq!(script.stage_title(0, Lang::En), "SECTOR ZERO");
    assert_eq!(script.stage_beat(0, 1, Lang::Pt), "Break the firewall.");
    assert_eq!(script.stages[1].alt_boss, "oboro");
}

#[test]
fn disabled_campaign_toml_is_valid_even_without_stages() {
    let raw = r#"
[campaign]
id = "off"
enabled = false
"#;
    let script = StoryScript::parse(raw).expect("disabled campaign parses");
    assert!(!script.campaign.enabled);
    assert!(script.stages.is_empty());
}

#[test]
fn invalid_or_empty_enabled_campaign_returns_error() {
    let empty_enabled = r#"
[campaign]
enabled = true
"#;
    assert!(StoryScript::parse(empty_enabled).is_err());
    assert!(StoryScript::parse("not = [valid toml").is_err());
}
