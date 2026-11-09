use super::{BOSS_ATTACK_FACTOR, BOSS_LIFE_FACTOR, FOE_START_LANES};
use crate::app::{App, Screen, Team};
use crate::session::Pilot;
use crate::story::text;
use ume_engine::ai::Ai;
use ume_engine::fighter::V2;

impl App {
    pub(crate) fn story_prefetch_stage(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            let Some(run) = self.story.as_ref() else {
                return;
            };
            let def = run.stage_def();
            let mut packs: Vec<String> = Vec::new();
            let ids = def
                .waves
                .iter()
                .flat_map(|w| w.iter().copied())
                .chain([def.boss, def.alt_boss]);
            for id in ids {
                if let Some(idx) = self.story_char(id) {
                    let pack = crate::web_sprites::pack_name(&self.reg.chars[idx]);
                    if !packs.contains(&pack) {
                        packs.push(pack);
                    }
                }
            }
            if !packs.is_empty() {
                crate::web_sprites::prefetch_roster(packs);
            }
        }
    }

    pub(crate) fn story_wave_next(&mut self, winner: usize) -> bool {
        if !self.story_is_playing() || winner != 0 {
            return false;
        }
        if self.story.as_ref().is_none_or(|run| run.is_boss_fight()) {
            return false;
        }
        self.story_win();
        true
    }

    pub(crate) fn story_advance_in_place(&mut self) -> bool {
        let Some(run) = self.story.clone() else {
            return false;
        };
        if self.session.is_none() {
            return false;
        }
        let foes = self.story_foes(&run);
        let Some(&first) = foes.first() else {
            return false;
        };
        #[cfg(target_arch = "wasm32")]
        if !foes
            .iter()
            .all(|&r| crate::web_sprites::ready(&self.reg.chars[r]).unwrap_or(false))
        {
            return false;
        }
        let hero_idx = self.sel.0;
        let mut foe_defs = Vec::with_capacity(foes.len());
        for (k, &r) in foes.iter().enumerate() {
            let dup = r == hero_idx || foes[..k].contains(&r);
            let pal = if dup { 2 } else { 1 };
            let Ok(d) = self.reg.chars[r].load(pal) else {
                return false;
            };
            foe_defs.push((d, pal));
        }
        self.sel.1 = first;
        self.team = Some(Team::new(vec![hero_idx], foes, true));
        let life_k = (self.fight_rules.life_percent as f32 / 100.0).max(0.1);
        let lvl = run.ai_level(self.ai_level);
        let Some(sess) = self.session.as_mut() else {
            return false;
        };
        let def = run.stage_def();
        let fight = run.fight_index();
        let center = crate::story::fight_center(def, fight);
        let prev_center = crate::story::fight_center(def, fight.saturating_sub(1));
        sess.over = None;
        sess.over_timer = 0.0;
        sess.m.round_state = 2;
        sess.m.round_time = 0;
        sess.m.timer = if sess.m.round_ticks > 0 {
            sess.m.round_ticks
        } else {
            99 * 60
        };
        sess.m.winner_side = -1;
        sess.m.wins = [0, 0];
        sess.m.match_over = false;
        sess.m.won_by_ko = false;
        sess.m.slow_mo = 0;
        sess.m.announce = None;
        sess.m.explods.clear();
        sess.m.projectiles.clear();
        sess.m.pause = None;
        sess.m.fighters.retain(|f| !f.is_helper && f.side == 0);
        sess.pilots.truncate(1);
        if let Some(Pilot::Cpu(ai)) = sess.pilots.first_mut() {
            ai.reset();
        }
        let hero_base_max = sess
            .m
            .fighters
            .first()
            .map(|f| (f.def.c("data.life", 1000.0) as f32) * life_k)
            .unwrap_or(1000.0);
        if let Some(hero) = sess.m.fighters.first_mut() {
            hero.life = (hero.life_max * run.life).clamp(1.0, hero.life_max);
            hero.alive = true;
            hero.destroyed = false;
            hero.score = 0.0;
            hero.score_total = 0.0;
            hero.received_damage = 0.0;
            hero.received_hits = 0;
            hero.combo_count = 0;
            hero.combo_damage = 0.0;
            hero.combo_target = None;
            hero.combo_timer = 0;
            hero.no_ctrl_ticks = 0;
            hero.pos.x = hero.pos.x.clamp(
                prev_center - crate::story::FIGHT_HALF_RANGE - 140.0,
                center + sess.stage.info.p1x - 160.0,
            );
            hero.pos.y = 0.0;
            hero.vel = V2::default();
            hero.ground_level = 0.0;
            hero.facing = 1.0;
            hero.hit_pause = 0;
            hero.hit_shake = 0;
            hero.juggle_used = 0;
            hero.targets.clear();
            hero.bind = None;
            hero.hitdef_active = false;
            hero.move_type = 'I';
            hero.state_type = 'S';
            hero.ctrl = true;
            hero.reset_passive_for_round();
            let hd = hero.def.clone();
            sess.m.change_state(0, 0, hd);
            ume_engine::ikemen::life::reset_for_round(&mut sess.m, 0);
            sess.m.fighters[0].ctrl = true;
        }
        sess.m.stage.start_x = center;
        sess.m.stage.p1x = center + sess.stage.info.p1x;
        sess.m.stage.p2x = center + sess.stage.info.p2x;
        sess.m.stage.bound_left = sess
            .m
            .stage
            .bound_left
            .min(prev_center - crate::story::FIGHT_HALF_RANGE);
        sess.m.stage.bound_right = center + crate::story::FIGHT_HALF_RANGE;
        sess.m.lanes.target_x = center;
        sess.m.lanes.bound_span = crate::story::FIGHT_HALF_RANGE;
        sess.m.lanes.show_foe_hud = run.is_boss_fight();
        sess.m.camera.x = sess.m.camera.x.min(center - 160.0);
        let n_foes = foe_defs.len();
        sess.m.team_simul = n_foes > 1;
        let ground_k = sess.m.ground_k;
        if sess.m.zoffset0 != 0.0 {
            sess.m.stage.zoffset = sess.m.zoffset0;
        }
        sess.m.set_zoom_for_teams(n_foes.max(1), ground_k);
        let gap = if n_foes <= 2 {
            52.0
        } else {
            130.0 / (n_foes - 1) as f32
        };
        for (k, (foe_def, pal)) in foe_defs.into_iter().enumerate() {
            let foe_lvl = if k == 0 { lvl } else { (lvl - 1).max(1) };
            let seed = if k == 0 { 99 } else { 31 + (7 + k) as u32 };
            let fi = sess.m.add_player(foe_def.clone(), 1, foe_lvl, pal);
            sess.pilots.push(Pilot::Cpu(Ai::new(foe_lvl, seed)));
            {
                let f = &mut sess.m.fighters[fi];
                f.pos = V2 {
                    x: sess.m.stage.p2x + gap * k as f32,
                    y: 0.0,
                };
                f.vel = V2::default();
                f.ground_level = 0.0;
                f.facing = -1.0;
                f.alive = true;
                f.lane =
                    FOE_START_LANES[k % FOE_START_LANES.len()].clamp(def.lane_far, def.lane_near);
                if run.is_boss_fight() {
                    f.life_max = f.life_max * life_k * BOSS_LIFE_FACTOR;
                    f.life = f.life_max;
                    f.attack_mul *= BOSS_ATTACK_FACTOR;
                } else {
                    let ratio = crate::story::villain_life_ratio(&f.def.id);
                    f.life_max = (hero_base_max * ratio).max(1.0);
                    f.life = f.life_max;
                }
            }
            sess.m.change_state(fi, 0, foe_def);
            sess.m.fighters[fi].ctrl = false;
        }
        let accent = def.accent;
        if run.is_boss_fight() {
            sess.vfx.banner(
                text::boss_banner(),
                &text::stage_title(run.stage).to_string(),
                accent,
            );
        } else {
            let label = text::wave_label(run.wave, def.waves.len(), false);
            sess.vfx.banner(
                &text::stage_label(run.stage),
                &format!("{label} · {}", text::go()),
                accent,
            );
        }
        let _ = Screen::Fight;
        true
    }

    pub(crate) fn run_life(&self) -> [f32; 2] {
        if self.story.is_some() && self.mode == crate::app::Mode::Story {
            self.story_life()
        } else {
            self.survival_life()
        }
    }

    pub(crate) fn run_win(&mut self) {
        if self.mode == crate::app::Mode::Story {
            self.story_win();
        } else {
            self.survival_win();
        }
    }

    pub(crate) fn run_loss(&mut self) {
        if self.mode == crate::app::Mode::Story {
            self.story_loss();
        } else {
            self.survival_loss();
        }
    }
}
