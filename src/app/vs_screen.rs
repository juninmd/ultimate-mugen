use super::*;

impl App {
    pub(super) async fn vs(&mut self, dt: f32) {
        let Screen::Vs { t, step } = &mut self.screen else {
            return;
        };
        *t += dt;
        let (t, step_v) = (*t, *step);
        // work for the step labelled on the previous frame (that frame is still on screen while we block)
        let mut step_now = step_v;
        let total = self.load_total();
        if t > dt * 1.5 && step_v < total {
            match self.load_step(step_v, false).await {
                Err(e) => {
                    self.abort_load(e);
                    return;
                }
                Ok(true) => {
                    step_now = step_v + 1;
                    if let Screen::Vs { step, .. } = &mut self.screen {
                        *step = step_now;
                    }
                }
                Ok(false) => {}
            }
        }
        let ready = step_now >= total;
        self.draw_versus(t);
        if !ready {
            let within = if step_now + 1 == total { self.warm_progress() } else { 0.0 };
            self.ui.progress_bar(
                440.0,
                75.0,
                400.0,
                (step_now as f32 + within) / total as f32,
                &self.load_label(step_now),
            );
        } else if t > 1.8
            || self.c.pressed(nav::OK | nav::START)
            || is_mouse_button_pressed(MouseButton::Left)
        {
            self.finish_fight().await;
        }
    }

    /// Synchronous variant (screenshot tour, rematch): runs every loading step back to back.
    /// The sprite warm-up is the one step that is paid over several frames here: it copies 1 MiB per
    /// frame, yielding between chunks, so a rematch or a netplay start does not freeze the window for
    /// the hundreds of milliseconds a full fighter upload takes.
    pub async fn start_fight(&mut self) {
        let total = self.load_total();
        for st in 0..total {
            if st + 1 == total {
                while !self.warm_step() {
                    next_frame().await;
                }
                continue;
            }
            loop {
                match self.load_step(st, false).await {
                    Ok(true) => break,
                    Ok(false) => {
                        self.ui.begin();
                        self.ui.loading_screen((st as f32 + 0.5) / total as f32, &self.load_label(st));
                        next_frame().await;
                    }
                    Err(e) => {
                        self.abort_load(e);
                        return;
                    }
                }
            }
        }
        self.finish_fight().await;
    }

    pub(super) async fn finish_fight(&mut self) {
        let Some(sess) = self.pending.sess.take() else {
            self.screen = Screen::Menu { cur: 0 };
            return;
        };
        let music = sess.stage.music;
        let bgm = sess.stage.bgm.clone().or_else(|| self.market.pick_track());
        // DualShock/DualSense lightbars: P1 blue, P2 red
        self.c.pads.set_light(0, [30, 90, 255]);
        self.c.pads.set_light(1, [255, 40, 60]);
        self.session = Some(sess);
        self.screen = Screen::Fight;
        self.audio.reset_bgm();
        if let Some(b) = bgm.as_ref() {
            self.audio.register_music_file("stage", b.0.clone(), b.1);
        }
        let custom = match bgm {
            Some(b) => self.audio.play_music_file(&b.0, b.1).await,
            None => false,
        };
        if !custom {
            self.audio.play_music(music).await;
        }
    }
}
