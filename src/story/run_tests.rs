use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stage_is_its_waves_then_one_boss() {
        let mut run = StoryRun::default();
        let waves = run.stage_def().waves.len();
        for _ in 0..waves {
            assert!(!run.is_boss_fight());
            assert_eq!(run.won_fight(0, 1.0), Next::Wave);
        }
        assert!(run.is_boss_fight());
        assert_eq!(run.foes("kaito"), vec!["bruno"]);
        assert_eq!(run.won_fight(0, 1.0), Next::StageCleared);
        assert_eq!((run.stage, run.wave), (1, 0));
    }

    #[test]
    fn clearing_a_stage_pays_a_life_bonus_and_records_the_tally() {
        let mut run = StoryRun::default();
        run.wave = run.stage_def().waves.len();
        let before = run.score;
        run.won_fight(0, 1.0);
        let tally = run.tally.expect("stage tally");
        assert!(tally.life > 0);
        assert_eq!(run.score - before, tally.total());
    }

    #[test]
    fn playing_as_the_boss_swaps_in_the_alternate() {
        let mut run = StoryRun::default();
        run.wave = run.stage_def().waves.len();
        assert_eq!(run.foes("bruno"), vec!["daichi"]);
    }

    #[test]
    fn the_last_boss_ends_the_story() {
        let mut run = StoryRun::default();
        run.stage = STAGES.len() - 1;
        run.wave = run.stage_def().waves.len();
        assert_eq!(run.won_fight(0, 1.0), Next::Finished);
        assert_eq!(run.phase, Phase::Victory);
    }

    #[test]
    fn continuing_spends_a_credit_and_restores_part_of_the_bar() {
        let mut run = StoryRun::default();
        run.life = 0.0;
        run.lost_fight(120);
        assert_eq!(run.phase, Phase::Defeated);
        run.continue_fight();
        assert_eq!(run.credits, STARTING_CREDITS - 1);
        assert!((run.life - CONTINUE_LIFE).abs() < 1e-6);
    }

    #[test]
    fn health_carries_over_and_recovers_a_little() {
        let mut run = StoryRun::default();
        run.won_fight(0, 0.3);
        assert!((run.life - (0.3 + WAVE_RECOVERY)).abs() < 1e-6);
        run.won_fight(0, 0.0);
        assert!(run.life >= MIN_CARRIED_LIFE);
    }

    #[test]
    fn longer_stages_have_a_strip_segment_per_fight() {
        for stage in STAGES.iter() {
            assert!(stage.waves.len() >= 4);
            let mid = super::super::super::stages::fight_center(stage, super::super::super::stages::fights_in(stage) / 2);
            assert!(mid.abs() <= super::super::super::stages::SEGMENT_WORLD);
        }
    }

    #[test]
    fn credits_run_out() {
        let mut run = StoryRun::default();
        for _ in 0..STARTING_CREDITS {
            run.continue_fight();
        }
        assert!(!run.can_continue());
    }

    #[test]
    fn difficulty_rises_with_stage_and_boss() {
        let mut run = StoryRun::default();
        let first = run.ai_level(4);
        run.stage = 3;
        assert!(run.ai_level(4) > first);
        run.wave = run.stage_def().waves.len();
        assert!(run.ai_level(4) <= 8);
    }

    #[test]
    fn every_stage_has_non_empty_waves_and_a_distinct_alternate_boss() {
        for stage in STAGES.iter() {
            assert!(stage.waves.iter().all(|w| !w.is_empty() && w.len() <= 4));
            assert_ne!(stage.boss, stage.alt_boss);
        }
    }

    #[test]
    fn wave_villains_have_around_ten_percent_of_protagonist_life_and_advance_right_to_boss() {
        for stage in STAGES.iter() {
            for wave in stage.waves {
                for &id in *wave {
                    let ratio = super::super::super::stages::villain_life_ratio(id);
                    assert!((0.08..=0.12).contains(&ratio), "{id} ratio {ratio} must be ~10%");
                }
            }
            let total = super::super::super::stages::fights_in(stage);
            for i in 1..total {
                let prev = super::super::super::stages::fight_center(stage, i - 1);
                let next = super::super::super::stages::fight_center(stage, i);
                assert!(next > prev, "fight {i} must advance right ({prev} -> {next})");
            }
        }
    }
}

#[cfg(test)]
mod strip_tests {
    use super::*;

    #[test]
    fn every_arena_strip_has_one_segment_per_fight() {
        for stage in STAGES.iter() {
            let strip = crate::stages::story_strip(stage.arena).expect("strip");
            assert_eq!(strip.w as usize, 836 * crate::story::fights_in(stage));
            assert_eq!(strip.h, 470);
        }
    }
}
