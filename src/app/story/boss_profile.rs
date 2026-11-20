use crate::app::App;
use crate::gfx::*;
use crate::story::StoryRun;
use crate::ui::*;

const NAME_Y: f32 = 592.0;
const NATIONALITY_Y: f32 = 614.0;
const BIO_Y: f32 = 638.0;
const BIO_LINE: f32 = 18.0;
const BIO_SIZE: f32 = 15.0;
const BIO_WIDTH: f32 = 760.0;
const BIO_LINES: usize = 3;
const FLAG_W: f32 = 24.0;

pub(crate) fn stage_boss(run: &StoryRun, player: &str) -> &'static str {
    run.stage_boss_id(player)
}

impl App {
    pub(crate) fn draw_story_boss_profile(&self, run: &StoryRun, accent: Color, reveal: f32) {
        let player = self.player_char_id();
        let Some(entry) = self
            .story_char(stage_boss(run, &player))
            .map(|i| &self.reg.chars[i])
        else {
            return;
        };
        let Some(profile) = crate::profile::of(entry) else {
            return;
        };
        let lang = crate::profile::lang();
        let boss = crate::i18n::tr3("CHEFE", "BOSS", "JEFE");
        let name = format!("{boss} · {}", entry.name.to_uppercase());
        let nationality = format!(
            "{}: {}",
            crate::profile::nationality_label(),
            profile.nationality.pick(lang)
        )
        .to_uppercase();
        self.ui.text(
            &name,
            VW / 2.0,
            NAME_Y,
            20.0,
            alpha(accent, reveal),
            Align::Center,
            true,
        );
        self.ui.text(
            &nationality,
            VW / 2.0,
            NATIONALITY_Y,
            13.0,
            Color::new(1.0, 1.0, 1.0, 0.75 * reveal),
            Align::Center,
            true,
        );
        let flag_x = (VW - self.ui.measure(&nationality, 13.0, true)) / 2.0 - FLAG_W - 8.0;
        self.ui.flag(
            &profile.country,
            flag_x,
            NATIONALITY_Y - 14.0,
            FLAG_W,
            reveal,
        );
        let ink = Color::new(0.85, 0.82, 0.94, 0.85 * reveal);
        let lines = crate::profile::wrap(profile.bio.pick(lang), BIO_WIDTH, |s| {
            self.ui.measure(s, BIO_SIZE, false)
        });
        for (n, line) in lines.iter().take(BIO_LINES).enumerate() {
            self.ui.text(
                line,
                VW / 2.0,
                BIO_Y + n as f32 * BIO_LINE,
                BIO_SIZE,
                ink,
                Align::Center,
                false,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stage_boss_swaps_when_the_player_picked_it() {
        let run = StoryRun::default();
        let def = run.stage_def();
        assert_eq!(stage_boss(&run, "kaito"), def.boss);
        assert_eq!(stage_boss(&run, def.boss), def.alt_boss);
    }
}
