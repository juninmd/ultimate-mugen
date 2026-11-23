use super::*;

#[test]
fn edited_script_survives_a_toml_round_trip() {
    let mut script = StoryScript::default_campaign();
    Rule::Credits.step(&mut script.campaign, 1);
    Rule::BossLife.step(&mut script.campaign, -1);
    set_beat(&mut script, 0, 1, Lang::Pt, "  Nova fala de teste  ");
    let text = to_toml(&script).unwrap();
    let back = StoryScript::parse(&text).unwrap();
    assert_eq!(back.campaign.credits, 4);
    assert_eq!(back.campaign.boss_life_factor, 1.4);
    assert_eq!(back.stage_beat(0, 1, Lang::Pt), "Nova fala de teste");
    assert_eq!(back.stages.len(), script.stages.len());
    assert_eq!(back.stages[0].accent, script.stages[0].accent);
}

#[test]
fn editing_one_language_keeps_the_others() {
    let mut script = StoryScript::default_campaign();
    let before_en = script.stage_beat(0, 1, Lang::En).to_string();
    set_beat(&mut script, 0, 1, Lang::Es, "Hola");
    assert_eq!(script.stage_beat(0, 1, Lang::Es), "Hola");
    assert_eq!(script.stage_beat(0, 1, Lang::En), before_en);
}

#[test]
fn plain_string_beats_expand_to_every_language_before_editing() {
    let raw = "[[stages]]\narena = \"a\"\nboss = \"b\"\nwaves = [[\"x\"]]\nbeats = [\"one\"]\n";
    let mut script = StoryScript::parse(raw).unwrap();
    set_beat(&mut script, 0, 2, Lang::En, "third");
    assert_eq!(script.stage_beat(0, 0, Lang::Pt), "one");
    assert_eq!(script.stage_beat(0, 2, Lang::En), "third");
    assert_eq!(script.stage_beat(0, 3, Lang::En), "one");
}

#[test]
fn rules_clamp_to_their_limits() {
    let mut meta = CampaignMeta::default();
    for _ in 0..20 {
        Rule::Credits.step(&mut meta, 1);
        Rule::WaveRecovery.step(&mut meta, 1);
        Rule::BossAttack.step(&mut meta, -1);
    }
    assert_eq!((meta.credits, meta.wave_recovery, meta.boss_attack_factor), (9, 1.0, 0.5));
}

#[test]
fn beat_text_is_capped() {
    let mut script = StoryScript::default_campaign();
    set_beat(&mut script, 0, 0, Lang::En, &"x".repeat(500));
    assert_eq!(script.stage_beat(0, 0, Lang::En).chars().count(), BEAT_MAX_CHARS);
}

#[test]
fn editing_a_stage_title_keeps_the_other_languages_and_caps_length() {
    let mut script = StoryScript::default_campaign();
    let pt = script.stage_title(0, Lang::Pt).to_string();
    set_title(&mut script, 0, Lang::En, &"T".repeat(80));
    assert_eq!(script.stage_title(0, Lang::En).chars().count(), TITLE_MAX_CHARS);
    assert_eq!(script.stage_title(0, Lang::Pt), pt);
}

#[test]
fn cycling_the_boss_wraps_and_never_leaves_both_bosses_equal() {
    let mut script = StoryScript::default_campaign();
    let roster: Vec<String> = ["a", "b", "c"].map(String::from).to_vec();
    script.stages[0].boss = "a".into();
    script.stages[0].alt_boss = "b".into();
    cycle_boss(&mut script, 0, &roster, 1);
    assert_eq!((script.stages[0].boss.as_str(), script.stages[0].alt_boss.as_str()), ("b", "a"));
    cycle_boss(&mut script, 0, &roster, -1);
    cycle_boss(&mut script, 0, &roster, -1);
    assert_eq!(script.stages[0].boss, "c");
    cycle_boss(&mut script, 0, &[], 1);
    assert_eq!(script.stages[0].boss, "c");
}

#[test]
fn foes_per_wave_clamp_and_keep_the_wave_count() {
    let mut script = StoryScript::default_campaign();
    let waves = script.stages[0].waves.len();
    for _ in 0..10 {
        adjust_foes(&mut script, 0, 1);
    }
    assert_eq!(foe_range(&script, 0), (FOES_MAX, FOES_MAX));
    for _ in 0..10 {
        adjust_foes(&mut script, 0, -1);
    }
    assert_eq!(foe_range(&script, 0), (FOES_MIN, FOES_MIN));
    assert_eq!(script.stages[0].waves.len(), waves);
}

#[test]
fn added_foes_reuse_the_waves_own_roster() {
    let mut script = StoryScript::default_campaign();
    let before = script.stages[0].waves[0].clone();
    adjust_foes(&mut script, 0, 1);
    let after = &script.stages[0].waves[0];
    assert_eq!(after.len(), before.len() + 1);
    assert!(after.iter().all(|id| before.contains(id)));
}
