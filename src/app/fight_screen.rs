use super::*;

impl App {
    pub(super) async fn fight(&mut self, dt: f32) {
        let replaying = self.replaying.is_some();
        let Some(s) = self.session.as_mut() else {
            self.screen = Screen::Menu { cur: 0 };
            return;
        };
        if replaying && is_key_pressed(KeyCode::Escape) {
            self.leave_replay();
            return;
        }
        if self.c.pressed(nav::AUX) {
            s.r.debug = !s.r.debug;
            self.debug_boxes = s.r.debug;
        }
        let pause_btn = if replaying {
            is_key_pressed(KeyCode::Space) || self.c.pressed(nav::START)
        } else {
            (self.c.pressed(nav::START) && !is_key_down(self.c.keys.slots[0][15]) && !is_key_pressed(KeyCode::Enter) && !is_key_pressed(KeyCode::KpEnter))
                || is_key_pressed(KeyCode::Escape)
                || is_key_pressed(KeyCode::P)
                || (!s.paused && is_key_pressed(KeyCode::Enter))
        };
        let (paused, accept_back) = pause_step(s.paused, pause_btn);
        if pause_btn {
            self.pause.reset();
            self.audio.sfx("select");
        }
        s.paused = paused;
        if replaying {
            if is_key_pressed(KeyCode::Key1) {
                s.set_speed(1.0);
            }
            if is_key_pressed(KeyCode::Key2) {
                s.set_speed(2.0);
            }
            if is_key_pressed(KeyCode::Key4) {
                s.set_speed(4.0);
            }
            if s.paused && (is_key_pressed(KeyCode::Period) || is_key_pressed(KeyCode::Right)) {
                s.step_frame(&mut self.c, &mut self.audio);
            }
        }
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if s.net.is_some() {
            s.paused = false;
        }
        // pause button (touch) top-center
        let pb = (VW / 2.0 - 22.0, 92.0, 44.0, 44.0);
        if !s.paused && self.c.touch.enabled && is_mouse_button_pressed(MouseButton::Left) {
            let p = self.ui.to_virtual(mouse_position().into());
            if p.x > pb.0 && p.x < pb.0 + pb.2 && p.y > pb.1 && p.y < pb.1 + pb.3 {
                s.paused = true;
            }
        }
        if is_key_pressed(KeyCode::Tab) && !s.paused && !replaying {
            s.switch_control();
        }
        s.update(dt, &mut self.c, &mut self.audio);
        s.draw(&mut self.g, &self.ui);
        if replaying {
            self.replay_bar();
        }
        if self.survival_is_playing() {
            self.survival_hud();
        }
        if self.story_is_playing() {
            self.story_hud();
        }
        self.c.touch.draw();
        if self.c.touch.enabled {
            self.ui
                .rect(pb.0, pb.1, pb.2, pb.3, Color::new(0.0, 0.0, 0.0, 0.35));
            self.ui.text(
                "II",
                VW / 2.0,
                pb.1 + 30.0,
                20.0,
                WHITE,
                Align::Center,
                true,
            );
        }
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let Some(n) = self.session.as_ref().and_then(|s| s.net.as_ref()) {
            self.net_overlay(n);
            if n.desyncs > 0 && !self.desync_reported {
                self.desync_reported = true;
                let (frame, checksum) = n.desync_at.unwrap_or((0, 0));
                let rec = self.session.as_ref().map(|s| s.rec.clone()).unwrap_or_default();
                match crate::replays::save_desync(
                    &crate::replays::dir(),
                    Some(&n.config),
                    frame,
                    checksum,
                    rec.len(),
                    &rec,
                ) {
                    Ok(p) => {
                        self.msg = format!(
                            "Desync: report at {}",
                            p.file_name()
                                .map(|f| f.to_string_lossy().into_owned())
                                .unwrap_or_default()
                        )
                    }
                    Err(e) => self.msg = e,
                }
            }
        }
        let s = self.session.as_mut().unwrap();
        if s.paused {
            if !replaying {
                self.pause_menu(accept_back).await;
            }
            return;
        }
        if let Some((w, t)) = s.over.map(|w| (w, s.over_timer)) {
            if !self.story_wave_next(w) && t > 1.0 && !self.team_next(w) {
                // keep the fight so it can be replayed (not for replays themselves or sequence team bouts)
                if self.replaying.is_none() && !self.team.as_ref().is_some_and(|t| !t.whole_squad())
                {
                    if let Some(s) = self.session.as_ref() {
                        self.last_fight = Some(Replay {
                            sel: self.sel,
                            mode: self.mode,
                            team: self.team.clone(),
                            rec: s.rec.clone(),
                            rules: self.fight_rules,
                            seed: s.seed,
                            life: self.run_life(),
                            pal: self.pal,
                            team_pal: self.team_pal.clone(),
                            net: s.netplay_config(),
                        });
                    }
                    self.archive_last_fight();
                }
                self.screen = Screen::Results { winner: w, cur: 0 };
            }
        }
    }

    /// Sequence mode: after a bout the winner keeps (most of) its life and the loser's next teammate steps in.
    /// Returns true when another bout was queued (false → the team match is over / not a sequence).
    pub(super) fn team_next(&mut self, w: usize) -> bool {
        let Some(t) = self.team.as_mut() else {
            return false;
        };
        if t.whole_squad() {
            return false;
        }
        let Some(s) = self.session.as_ref() else {
            return false;
        };
        let ratio =
            s.m.roots()
                .find(|f| f.side == w)
                .map(|f| (f.life / f.life_max).clamp(0.0, 1.0))
                .unwrap_or(1.0);
        // shared team meter carries over to the next fighter
        for side in 0..2 {
            if let Some(f) = s.m.roots().find(|f| f.side == side) {
                t.power[side] = f.power;
            }
        }
        let lose = 1 - w;
        if let Some(k) = t.ko[lose].get_mut(t.idx[lose]) {
            *k = true;
        }
        t.idx[lose] += 1;
        t.life[lose] = 1.0;
        t.life[w] = (ratio + 0.12).min(1.0);
        if t.idx[lose] >= t.members[lose].len() {
            return false;
        }
        self.sel = (t.members[0][t.idx[0]], t.members[1][t.idx[1]], self.sel.2);
        self.audio.stop_music();
        self.session = None;
        self.screen = Screen::Vs { t: 0.0, step: 0 };
        true
    }

    pub(super) fn reset_team(&mut self) {
        if let Some(t) = self.team.as_mut() {
            *t = Team {
                tag: t.tag,
                ..Team::new(t.members[0].clone(), t.members[1].clone(), t.simul)
            };
            self.sel = (t.members[0][0], t.members[1][0], self.sel.2);
        }
    }
}
