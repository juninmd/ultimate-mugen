use crate::app::vs_epic::Side;
use crate::app::App;
use crate::gfx::*;
use crate::story::text;
use crate::ui::*;

fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}

impl App {
    pub(crate) fn story_shows_stage_card(&self) -> bool {
        self.story_is_playing() && self.story.as_ref().is_some_and(|run| !run.is_boss_fight())
    }

    pub(crate) fn draw_story_stage_card(&self, t: f32) {
        let Some(run) = self.story.as_ref() else {
            return;
        };
        let accent = run.stage_def().accent;
        let accent_color = Color::new(accent[0], accent[1], accent[2], 1.0);
        let hero = &self.reg.chars[self.sel.0];
        let slide = (1.0 - ease_out(t / 0.5)) * 760.0;
        self.vs_backdrop(
            [
                Side { col: crate::app::rgb(hero.color), off: -slide },
                Side { col: accent_color, off: slide },
            ],
            VW / 2.0,
            110.0,
        );
        let reveal = ease_out((t - 0.2) / 0.4);
        let alpha = reveal.clamp(0.0, 1.0);
        let waves = run.stage_def().waves.len();
        self.ui.adapted_glass(VW / 2.0 - 340.0, 214.0, 680.0, 220.0, accent_color, alpha);
        self.ui.text(
            &text::stage_label(run.stage),
            VW / 2.0,
            262.0,
            26.0,
            Color::new(accent[0], accent[1], accent[2], alpha),
            Align::Center,
            true,
        );
        self.ui.text_bold(
            text::stage_title(run.stage),
            VW / 2.0,
            338.0,
            58.0,
            Color::new(1.0, 1.0, 1.0, alpha),
            BLACK,
            Align::Center,
        );
        self.ui.text(
            &text::wave_label(run.wave, waves, false),
            VW / 2.0,
            396.0,
            24.0,
            Color::new(0.85, 0.82, 0.94, alpha),
            Align::Center,
            false,
        );
        self.ui.text(
            &hero.name.to_uppercase(),
            VW / 2.0,
            424.0,
            16.0,
            Color::new(1.0, 0.8, 0.2, alpha),
            Align::Center,
            true,
        );
    }

    pub(crate) fn draw_story_boss_tag(&self, t: f32) {
        let Some(run) = self.story.as_ref().filter(|_| self.story_is_playing()) else {
            return;
        };
        let accent = run.stage_def().accent;
        let alpha = ease_out((t - 0.5) / 0.4).clamp(0.0, 1.0);
        let color = Color::new(accent[0], accent[1], accent[2], alpha);
        self.ui.adapted_glass(VW / 2.0 - 260.0, 600.0, 520.0, 84.0, color, alpha);
        self.ui.text(text::boss_banner(), VW / 2.0, 634.0, 30.0, color, Align::Center, true);
        self.ui.text(
            text::stage_title(run.stage),
            VW / 2.0,
            664.0,
            18.0,
            Color::new(0.85, 0.82, 0.94, alpha),
            Align::Center,
            false,
        );
    }
}
