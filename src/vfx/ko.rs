use super::quality;
use super::Vfx;
use ume_engine::matchs::Match;

impl Vfx {
    pub(super) fn ko_burst(&mut self, m: &Match, loser_side: usize) {
        let Some((x, y, _)) = Self::chest(m, loser_side) else {
            return;
        };
        let red = [1.0, 0.25, 0.3];
        let feet = y + 30.0;
        self.ko_focus = (x, y);
        self.ring(x, y, 110.0, 0.8, 3.0, red);
        self.ring(x, y, 190.0, 1.0, 2.0, [1.0, 0.6, 0.55]);
        self.ring(x, feet, 150.0, 0.9, 2.2, [1.0, 0.9, 0.85]);
        self.glow(x, y, 54.0, 0.45, red);
        self.glow(x, y, 22.0, 0.3, [1.0, 1.0, 1.0]);
        self.light(x, y, 200.0, 0.7, red);
        self.spark_burst(
            x,
            y,
            1.0,
            quality::count(30),
            9.0,
            [1.0, 0.45, 0.3],
            std::f32::consts::PI,
        );
        self.spark_burst(
            x,
            y,
            1.0,
            quality::count(10),
            12.0,
            [1.0, 0.95, 0.85],
            std::f32::consts::PI,
        );
        self.slashes(x, y, 1.0, quality::count(8), 52.0, [1.0, 0.85, 0.8]);
        self.debris(x, y + 12.0, quality::count(10), [0.7, 0.6, 0.55]);
        if quality::level() >= 2 {
            self.cracks(x, feet, 1.0, 60.0);
            self.cracks(x, feet, -1.0, 60.0);
        }
    }
}
