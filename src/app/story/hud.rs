use crate::app::App;
use crate::gfx::*;
use crate::story::text;
use crate::ui::*;
use ume_engine::stage_objects::StageObjects;

const TARGET_BAR: (f32, f32, f32, f32) = (700.0, 34.0, 540.0, 20.0);

impl App {
    pub(crate) fn story_hud(&self) {
        let Some(run) = self.story.as_ref() else {
            return;
        };
        let objects = self.session.as_ref().map(|s| &s.m.objects);
        let line = match objects.filter(|_| run.bonus) {
            Some(o) => format!(
                "{} · {} {}",
                text::bonus_hud(o.intact_count(), o.breakables.len()),
                run.score + self.story_fight_score(),
                crate::i18n::tr("PONTOS", "POINTS")
            ),
            None => text::hud(
                run.stage,
                run.wave,
                run.stage_def().waves.len(),
                run.is_boss_fight(),
                run.score,
            ),
        };
        self.ui.rect(
            VW / 2.0 - 230.0,
            652.0,
            460.0,
            44.0,
            Color::new(0.02, 0.01, 0.06, 0.6),
        );
        self.ui.text(
            &line,
            VW / 2.0,
            674.0,
            17.0,
            hex("#ffcc33"),
            Align::Center,
            true,
        );
        self.ui.text(
            &text::credits_line(run.credits),
            VW / 2.0,
            690.0,
            12.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
        if let Some(o) = objects.filter(|_| run.bonus) {
            self.story_bonus_target_bar(o);
        }
    }

    fn story_bonus_target_bar(&self, objects: &StageObjects) {
        let total: f32 = objects.breakables.iter().map(|b| b.life_max).sum();
        let left: f32 = objects.breakables.iter().map(|b| b.life.max(0.0)).sum();
        let ratio = if total > 0.0 {
            (left / total).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (x, y, w, h) = TARGET_BAR;
        let solid = objects.breakables.iter().any(|b| b.solid);
        self.ui.rect(
            x - 6.0,
            y - 26.0,
            w + 12.0,
            h + 36.0,
            Color::new(0.04, 0.0, 0.06, 0.78),
        );
        self.ui
            .rect(x - 6.0, y - 26.0, w + 12.0, 3.0, hex("#ffcc33"));
        self.ui.text(
            text::bonus_banner(),
            x,
            y - 7.0,
            15.0,
            hex("#ffe08a"),
            Align::Left,
            true,
        );
        self.ui.text(
            text::bonus_goal(solid),
            x + w,
            y - 7.0,
            15.0,
            WHITE,
            Align::Right,
            true,
        );
        self.ui.rect(
            x - 2.0,
            y - 2.0,
            w + 4.0,
            h + 4.0,
            Color::new(0.0, 0.0, 0.0, 0.9),
        );
        self.ui.rect(x, y, w, h, Color::new(0.2, 0.12, 0.04, 1.0));
        let fill = if ratio < 0.3 {
            hex("#ff5a3d")
        } else {
            hex("#ffb02e")
        };
        self.ui.rect(x, y, w * ratio, h, fill);
        self.ui
            .rect(x, y, w * ratio, h * 0.4, Color::new(1.0, 1.0, 1.0, 0.3));
    }
}
