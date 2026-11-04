//! Replay browser, on-disk persistence and playback controls.
use crate::app::{App, Replay, Screen, Team};
use crate::controls::nav;
use crate::gfx::*;
use crate::replays::{self, StoredReplay, StoredRules};
use crate::settings::Rules;
use crate::ui::*;

pub(crate) const KEEP: usize = 64;

fn rules_of(r: StoredRules) -> Rules {
    Rules { rounds_to_win: r.rounds_to_win, round_ticks: r.round_ticks, life_percent: r.life_percent, single_round: r.single_round, classic_pushbox: r.classic_pushbox, team_life_share: r.team_life_share, game_speed: r.game_speed }
}

fn stored_rules(r: Rules) -> StoredRules {
    StoredRules { rounds_to_win: r.rounds_to_win, round_ticks: r.round_ticks, life_percent: r.life_percent, single_round: r.single_round, classic_pushbox: r.classic_pushbox, team_life_share: r.team_life_share, game_speed: r.game_speed }
}

impl App {
    pub(crate) fn refresh_replays(&mut self) {
        let dir = replays::dir();
        self.replay_list = replays::list(&dir);
        self.replay_problems = replays::problems(&dir).into_iter().map(|(p, e)| format!("{}: {e}", p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default())).collect();
    }

    fn fight_label(&self, r: &Replay) -> String {
        let name = |i: usize| self.reg.chars.get(i).map(|c| c.name.clone()).unwrap_or_else(|| "?".into());
        match &r.team {
            Some(t) => format!("{} vs {}", name(t.members[0].first().copied().unwrap_or(r.sel.0)), name(t.members[1].first().copied().unwrap_or(r.sel.1))),
            None => format!("{} vs {}", name(r.sel.0), name(r.sel.1)),
        }
    }

    pub(crate) fn stored_replay(&self, r: &Replay, label: String) -> Option<StoredReplay> {
        let chars = [self.reg.chars.get(r.sel.0)?.id.clone(), self.reg.chars.get(r.sel.1)?.id.clone()];
        let stage = self.reg.stages.get(r.sel.2)?.id.clone();
        let teams = r.team.as_ref().map(|t| [0, 1].map(|sd| t.members[sd].iter().filter_map(|&i| self.reg.chars.get(i).map(|c| c.id.clone())).collect::<Vec<_>>()));
        Some(StoredReplay {
            label, saved_at: replays::now(), mode: r.mode.key().to_string(), chars, stage,
            palettes: r.pal, member_palettes: r.team_pal.clone(), seed: r.seed, life: r.life, teams,
            simul: r.team.as_ref().is_some_and(|t| t.simul), tag: r.team.as_ref().is_some_and(|t| t.tag),
            rules: stored_rules(r.rules), netplay: r.net.clone(), runs: StoredReplay::from_stream(&r.rec),
        })
    }

    pub(crate) fn replay_from_stored(&self, s: &StoredReplay) -> Result<Replay, String> {
        let find = |id: &str| self.reg.chars.iter().position(|c| c.id.eq_ignore_ascii_case(id) || c.name.eq_ignore_ascii_case(id));
        let first = find(&s.chars[0]).ok_or_else(|| format!("Replay: personagem não encontrado ({})", s.chars[0]))?;
        let second = find(&s.chars[1]).ok_or_else(|| format!("Replay: personagem não encontrado ({})", s.chars[1]))?;
        let stage = self.reg.stages.iter().position(|st| st.id.eq_ignore_ascii_case(&s.stage)).ok_or_else(|| format!("Replay: cenário não encontrado ({})", s.stage))?;
        let team = match &s.teams {
            None => None,
            Some(t) => {
                let mut members = [vec![], vec![]];
                for side in 0..2 {
                    for id in &t[side] { members[side].push(find(id).ok_or_else(|| format!("Replay: personagem não encontrado ({id})"))?); }
                }
                Some(if s.tag { Team::tagged(members[0].clone(), members[1].clone()) } else { Team::new(members[0].clone(), members[1].clone(), s.simul) })
            }
        };
        Ok(Replay { sel: (first, second, stage), mode: crate::app::Mode::from_key(&s.mode), team, rec: s.stream(), rules: rules_of(s.rules), seed: s.seed, life: s.life, pal: s.palettes, team_pal: s.member_palettes.clone(), net: s.netplay.clone() })
    }

    /// Write the just-finished fight to `replays/` (desktop). Older files are pruned so the folder stays small.
    pub(crate) fn archive_last_fight(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(r) = self.last_fight.clone() else { return };
            let label = self.fight_label(&r);
            let Some(stored) = self.stored_replay(&r, label) else { return };
            let dir = replays::dir();
            if let Err(e) = replays::save(&dir, &stored) { self.msg = e; return; }
            replays::prune(&dir, KEEP);
        }
    }

    pub(crate) fn leave_replay(&mut self) {
        self.audio.stop_music();
        self.session = None;
        self.replaying = None;
        self.team = None;
        self.refresh_replays();
        self.screen = Screen::Replays { cur: 0 };
    }

    fn play_stored(&mut self, index: usize) {
        let Some(entry) = self.replay_list.get(index) else { return };
        match self.replay_from_stored(&entry.replay) {
            Ok(r) => {
                self.mode = r.mode;
                self.sel = r.sel;
                self.team = r.team.clone();
                self.pal = r.pal;
                self.team_pal = r.team_pal.clone();
                self.arcade = None;
                self.replaying = Some(r);
                self.audio.stop_music();
                self.screen = Screen::Vs { t: 0.0, step: 0 };
            }
            Err(e) => self.msg = e,
        }
    }

    /// Bottom bar during playback: speed, progress and the keys (SPACE pauses, `.` steps a frame, ESC leaves).
    pub(crate) fn replay_bar(&self) {
        let total = self.replaying.as_ref().map(|r| r.rec.len()).unwrap_or(0);
        let (speed, paused, tick) = self.session.as_ref().map(|s| (s.speed(), s.paused, s.rec.len().min(total))).unwrap_or((1.0, false, 0));
        self.ui.rect(VW / 2.0 - 320.0, VH - 52.0, 640.0, 36.0, Color::new(0.02, 0.01, 0.06, 0.62));
        let text = crate::i18n::tf("replays.bar", &[("{speed}", &format!("{speed:.0}")), ("{tick}", &tick.to_string()), ("{total}", &total.to_string()), ("{action}", crate::i18n::t(if paused { "replays.resume" } else { "replays.pause" }))]);
        self.ui.text(&text, VW / 2.0, VH - 28.0, 15.0, hex("#d8d0f0"), Align::Center, false);
        if paused { self.ui.text(crate::i18n::t("replays.paused"), VW / 2.0, VH - 62.0, 22.0, hex("#ffcc33"), Align::Center, true); }
    }

    pub(crate) fn replays_screen(&mut self) {
        let Screen::Replays { mut cur } = self.screen else { return };
        self.ui.background();
        self.ui.motes(30, 0.8);
        self.ui.text(crate::i18n::t("replays.title"), 40.0, 56.0, 34.0, hex("#ffcc33"), Align::Left, true);
        if self.replay_list.is_empty() {
            self.ui.text(crate::i18n::t("replays.empty"), VW / 2.0, 300.0, 26.0, hex("#a49cc0"), Align::Center, false);
            self.ui.text(crate::i18n::t("replays.empty_hint"), VW / 2.0, 340.0, 20.0, hex("#a49cc0"), Align::Center, false);
        }
        let count = self.replay_list.len();
        cur = cur.min(count.saturating_sub(1));
        if let Some(buf) = &mut self.replay_rename {
            buf.push_str(&typed_text());
            if is_key_pressed(KeyCode::Backspace) { buf.pop(); }
            let (x, y, w, h) = (240.0, 230.0, 800.0, 260.0);
            self.ui.rect(0.0, 0.0, VW, VH, Color::new(0.0, 0.0, 0.0, 0.7));
            self.ui.rect(x, y, w, h, hex("#1a1030"));
            self.ui.rect_lines(x, y, w, h, 2.0, hex("#ff4d6d"));
            self.ui.text(crate::i18n::t("replays.rename_title"), x + 30.0, y + 46.0, 26.0, hex("#ffcc33"), Align::Left, true);
            self.ui.rect(x + 30.0, y + 84.0, w - 60.0, 48.0, Color::new(1.0, 1.0, 1.0, 0.1));
            let caret = if (self.ui.t * 2.0) as i32 % 2 == 0 { "|" } else { "" };
            let shown: String = buf.chars().take(64).collect();
            self.ui.text(&format!("{shown}{caret}"), x + 44.0, y + 118.0, 24.0, WHITE, Align::Left, false);
            self.ui.text(crate::i18n::t("replays.confirm"), x + 30.0, y + 176.0, 17.0, hex("#a49cc0"), Align::Left, false);
            let ok = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) || self.clicked_rect(x + w - 330.0, y + h - 64.0, 150.0, 44.0);
            let cancel = is_key_pressed(KeyCode::Escape) || self.clicked_rect(x + w - 170.0, y + h - 64.0, 140.0, 44.0);
            self.ui.rect(x + w - 330.0, y + h - 64.0, 150.0, 44.0, alpha(hex("#2ecc71"), 0.5));
            self.ui.text(crate::i18n::t("common.ok"), x + w - 255.0, y + h - 36.0, 20.0, WHITE, Align::Center, true);
            self.ui.rect(x + w - 170.0, y + h - 64.0, 140.0, 44.0, alpha(hex("#ff4d6d"), 0.5));
            self.ui.text(crate::i18n::t("common.cancel"), x + w - 100.0, y + h - 36.0, 20.0, WHITE, Align::Center, true);
            let finish = if ok { Some(true) } else if cancel { Some(false) } else { None };
            match finish {
                Some(commit) => {
                    let label = self.replay_rename.take().unwrap_or_default();
                    if commit { if let Some(e) = self.replay_list.get(cur) { match replays::rename(&e.path, &label) { Ok(_) => { self.refresh_replays(); self.msg = crate::i18n::t("replays.renamed").into(); } Err(m) => self.msg = m } } }
                    self.screen = Screen::Replays { cur };
                }
                None => self.screen = Screen::Replays { cur },
            }
            return;
        }
        for (i, e) in self.replay_list.iter().enumerate().take(12) {
            let y = 100.0 + i as f32 * 46.0;
            let selected = i == cur;
            if self.hover_rect(50.0, y, 1180.0, 40.0) && mouse_delta_position().length() > 0.0 { cur = i; }
            if self.clicked_rect(50.0, y, 1180.0, 40.0) { cur = i; self.play_stored(i); return; }
            self.ui.rect(50.0, y, 1180.0, 40.0, if selected { alpha(hex("#ff4d6d"), 0.5) } else { Color::new(1.0, 1.0, 1.0, if i % 2 == 0 { 0.07 } else { 0.03 }) });
            self.ui.text(&format!("{:02}", i + 1), 66.0, y + 27.0, 18.0, hex("#a49cc0"), Align::Left, false);
            self.ui.text(&e.replay.label.chars().take(52).collect::<String>(), 110.0, y + 27.0, 20.0, WHITE, Align::Left, true);
            self.ui.text(replays::date_label(e.replay.saved_at).as_str(), 720.0, y + 27.0, 16.0, hex("#a49cc0"), Align::Left, false);
            self.ui.text(&format!("{:.0}s", e.replay.seconds()), 900.0, y + 27.0, 16.0, hex("#a49cc0"), Align::Left, false);
            self.ui.text(crate::app::Mode::from_key(&e.replay.mode).label(), 980.0, y + 27.0, 16.0, hex("#22e8ff"), Align::Left, false);
        }
        if count > 12 { self.ui.text(&format!("{} replays · use ↑/↓", count), VW / 2.0, 660.0, 16.0, hex("#a49cc0"), Align::Center, false); }
        if let Some(first) = self.replay_problems.first() { self.ui.text(&format!("{} replay(s) ilegível(is): {first}", self.replay_problems.len()), 60.0, 686.0, 15.0, hex("#ff9e6d"), Align::Left, false); }
        let buttons = [(crate::i18n::t("replays.play"), "Enter"), (crate::i18n::t("replays.rename"), "R"), (crate::i18n::t("replays.delete"), "Del"), (crate::i18n::t("replays.sort"), "S"), (crate::i18n::t("replays.back"), "Esc")];
        let mut action = None;
        for (i, (label, key)) in buttons.iter().enumerate() {
            let x = 60.0 + i as f32 * 236.0;
            let hover = self.hover_rect(x, 668.0, 220.0, 44.0);
            self.ui.rect(x, 668.0, 220.0, 44.0, if hover { alpha(hex("#ff4d6d"), 0.7) } else { Color::new(1.0, 1.0, 1.0, 0.09) });
            self.ui.text(label, x + 110.0, 690.0, 17.0, WHITE, Align::Center, true);
            self.ui.text(key, x + 110.0, 706.0, 11.0, Color::new(1.0, 1.0, 1.0, 0.6), Align::Center, false);
            if self.clicked_rect(x, 668.0, 220.0, 44.0) { action = Some(i); }
        }
        if self.replay_confirm { self.ui.text(crate::i18n::t("replays.confirm_delete"), VW / 2.0, 650.0, 17.0, hex("#ff4d6d"), Align::Center, true); }
        if self.c.pressed(nav::DOWN) && count > 0 { cur = (cur + 1) % count; self.replay_confirm = false; }
        if self.c.pressed(nav::UP) && count > 0 { cur = (cur + count - 1) % count; self.replay_confirm = false; }
        if is_key_pressed(KeyCode::R) && count > 0 { action = Some(1); }
        if is_key_pressed(KeyCode::Delete) && count > 0 { action = Some(2); }
        if is_key_pressed(KeyCode::S) { action = Some(3); }
        if self.c.pressed(nav::OK | nav::START) && count > 0 { action = Some(0); }
        match action {
            Some(0) if count > 0 => self.play_stored(cur),
            Some(1) if count > 0 => { self.replay_rename = Some(self.replay_list[cur].replay.label.clone()); self.replay_confirm = false; }
            Some(2) if count > 0 => {
                if self.replay_confirm {
                    let path = self.replay_list[cur].path.clone();
                    match replays::delete(&path) { Ok(_) => { self.refresh_replays(); self.msg = crate::i18n::t("replays.deleted").into(); } Err(e) => self.msg = e }
                    self.replay_confirm = false;
                    self.screen = Screen::Replays { cur: 0 };
                    return;
                }
                self.replay_confirm = true;
            }
            Some(3) => {
                self.replay_by_name = !self.replay_by_name;
                if self.replay_by_name { self.replay_list.sort_by(|a, b| a.replay.label.to_lowercase().cmp(&b.replay.label.to_lowercase())); }
                else { self.refresh_replays(); }
                self.replay_confirm = false;
            }
            _ => {}
        }
        if self.c.pressed(nav::BACK) || is_key_pressed(KeyCode::Escape) { self.screen = Screen::Menu { cur: 13 }; return; }
        self.screen = Screen::Replays { cur };
    }
}
