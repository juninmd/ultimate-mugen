use super::*;
use crate::i18n::tr3;
use crate::trials::{self, Goal, Sample, Tracker};

const PANEL: Color = Color::new(0.02, 0.01, 0.06, 0.6);

fn goal_text(goal: Goal) -> String {
    match goal {
        Goal::Hits(n) => format!("{n} {}", tr3("ACERTOS", "HITS", "GOLPES")),
        Goal::Damage(share) => format!("{}% {}", (share * 100.0).round(), tr3("DE DANO", "DAMAGE", "DE DAÑO")),
        Goal::SuperHits(n) => format!("{n}+ {}", tr3("ACERTOS COM SUPER", "HITS WITH A SUPER", "GOLPES CON SUPER")),
        Goal::CleanHits(n) => format!("{n}+ {}", tr3("ACERTOS SEM BARRA", "HITS, NO METER", "GOLPES SIN BARRA")),
    }
}

impl App {
    pub(super) fn trials_step(&mut self, dt: f32) {
        if self.mode != Mode::Training || self.replaying.is_some() {
            return;
        }
        let Some(s) = self.session.as_ref().filter(|s| !s.paused) else { return };
        let mut roots = s.m.roots();
        let (Some(me), Some(rival)) = (roots.find(|f| f.side == 0), s.m.roots().find(|f| f.side == 1)) else { return };
        let name = me.def.display_name.clone();
        let sample = Sample { hits: me.combo_count, damage: me.combo_damage, life_max: rival.life_max, power: me.power };
        if self.trials.as_ref().is_none_or(|(n, _)| *n != name) {
            self.trials = Some((name.clone(), Tracker::with_progress(trials::load(&name))));
        }
        let Some((_, tracker)) = self.trials.as_mut() else { return };
        tracker.tick(dt);
        if tracker.observe(tracker.demo.unwrap_or(sample)).is_some() {
            trials::save(&name, tracker.cleared);
            self.audio.sfx("select");
        }
    }

    pub(super) fn trials_draw(&self) {
        let (Some((_, t)), Mode::Training) = (self.trials.as_ref(), self.mode) else { return };
        if self.replaying.is_some() || self.session.as_ref().is_none_or(|s| s.paused) {
            return;
        }
        let (w, h) = (330.0, 54.0);
        let (x, y) = ((VW - w) / 2.0, if self.c.touch.enabled { 146.0 } else { 100.0 });
        self.ui.rect(x, y, w, h, PANEL);
        let title = format!("{} {}/{}", tr3("DESAFIO", "TRIAL", "DESAFÍO"), (t.cleared + 1).min(trials::LADDER.len()), trials::LADDER.len());
        let Some(goal) = t.goal() else {
            self.ui.text(tr3("TODOS OS DESAFIOS CONCLUÍDOS", "ALL TRIALS CLEARED", "TODOS LOS DESAFÍOS COMPLETADOS"), x + 12.0, y + 33.0, 16.0, hex("#7dffb0"), Align::Left, true);
            return;
        };
        self.ui.text(&title, x + 12.0, y + 20.0, 13.0, hex("#9aa4d0"), Align::Left, false);
        self.ui.text(&goal_text(goal), x + 12.0, y + 42.0, 19.0, hex("#ffcc33"), Align::Left, true);
        let fill = match goal {
            Goal::Hits(n) | Goal::SuperHits(n) | Goal::CleanHits(n) => t.live_hits() as f32 / n as f32,
            Goal::Damage(share) => t.live_share() / share,
        };
        self.ui.rect(x + 12.0, y + h - 7.0, w - 24.0, 3.0, Color::new(1.0, 1.0, 1.0, 0.15));
        self.ui.rect(x + 12.0, y + h - 7.0, (w - 24.0) * fill.clamp(0.0, 1.0), 3.0, hex("#ffcc33"));
        if t.flash > 0.0 {
            let a = t.flash.min(1.0);
            self.ui.text(tr3("DESAFIO CUMPRIDO!", "TRIAL CLEAR!", "¡DESAFÍO COMPLETADO!"), VW / 2.0, y + 96.0, 34.0, Color::new(0.5, 1.0, 0.7, a), Align::Center, true);
        }
    }
}
