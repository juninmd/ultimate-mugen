use super::quality;
use super::Vfx;
use ume_engine::matchs::Match;

const CYAN: [f32; 3] = [0.35, 0.85, 1.0];
const TEAL: [f32; 3] = [0.4, 0.95, 0.75];
const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
const EMBER_PERIOD: u32 = 3;

impl Vfx {
    pub(super) fn on_rush(&mut self, fighter: u32, guarded: bool, m: &Match) {
        let col = if guarded { TEAL } else { CYAN };
        if let Some(f) = m.fighters.iter().find(|f| f.id == fighter) {
            let (x, y, _) = Self::chest_of(f);
            let feet = f.pos.y + f.lane;
            for (i, radius) in [60.0f32, 120.0, 190.0].iter().enumerate() {
                self.ring(
                    x,
                    y,
                    *radius,
                    0.35 + i as f32 * 0.12,
                    2.6 - i as f32 * 0.7,
                    col,
                );
            }
            self.ring(f.pos.x, feet - 1.0, 150.0, 0.5, 2.0, WHITE);
            self.glow(x, y, 70.0, 0.45, col);
            self.glow(x, y, 26.0, 0.3, WHITE);
            self.light(x, y, 210.0, 0.7, col);
            self.spark_burst(
                x,
                y,
                1.0,
                quality::count(26),
                9.0,
                col,
                std::f32::consts::PI,
            );
            self.slashes(x, y, 1.0, 6, 46.0, col);
            self.dust(f.pos.x, feet, quality::count(8), 2.2, 0.6, [0.8, 0.85, 0.9]);
            self.tag(
                x,
                y - 46.0,
                crate::i18n::tr3("CANCELAMENTO RUSH", "RUSH CANCEL", "CANCELACIÓN RUSH"),
                col,
            );
        }
        self.sfx.extend(["meter", "dash"]);
        self.flash_add(0.3);
        self.chroma = self.chroma.max(0.5);
        self.radial = self.radial.max(0.55);
        self.trauma = (self.trauma + 0.18).min(0.7);
        self.lines = self.lines.max(0.7);
        self.lines_col = col;
    }

    pub(super) fn overdrive_embers(&mut self, m: &Match) {
        if !self.tick.is_multiple_of(EMBER_PERIOD) {
            return;
        }
        for f in m
            .fighters
            .iter()
            .filter(|f| !f.is_helper && f.alive && f.rush.overdrive > 0)
        {
            let ratio = f.rush.overdrive_ratio();
            let (px, py) = (
                f.pos.x + self.rr(-16.0, 16.0),
                f.pos.y + f.lane + self.rr(-90.0, -4.0),
            );
            let (vx, vy) = (self.rr(-0.25, 0.25), -self.rr(1.0, 2.6));
            let (life, size) = (self.rr(0.35, 0.8) * (0.4 + ratio), self.rr(0.6, 1.4));
            self.ember(px, py, vx, vy, -0.01, life, size, CYAN);
        }
    }
}
