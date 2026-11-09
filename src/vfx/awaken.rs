use super::quality;
use super::{Kind, Vfx, P};
use ume_engine::matchs::Match;

const TAU: f32 = std::f32::consts::TAU;

impl Vfx {
    pub(super) fn on_super(&mut self, fighter: u32, m: &Match) {
        self.super_t = 1.0;
        self.super_owner = fighter;
        if let Some(f) = m.fighters.iter().find(|f| f.id == fighter) {
            let (x, y, _) = Self::chest_of(f);
            let feet = f.pos.y + f.lane;
            let gold = [1.0, 0.82, 0.35];
            self.ring(x, y, 70.0, 0.5, 2.4, [1.0, 0.9, 0.4]);
            self.ring(x, y, 130.0, 0.65, 1.6, gold);
            self.ring(f.pos.x, feet - 1.0, 150.0, 0.7, 2.0, [1.0, 0.95, 0.7]);
            self.glow(x, y, 40.0, 0.35, gold);
            self.glow(x, y, 18.0, 0.2, [1.0, 1.0, 0.95]);
            self.spark_burst(
                x,
                y,
                1.0,
                quality::count(30),
                9.0,
                gold,
                std::f32::consts::PI,
            );
            self.debris(f.pos.x, feet, quality::count(6), [0.75, 0.68, 0.6]);
            self.light(x, y, 180.0, 0.6, gold);
            self.pillar(f.pos.x, feet, gold, quality::count(16));
            if quality::level() >= 2 {
                self.cracks(f.pos.x, feet, 1.0, 54.0);
                self.cracks(f.pos.x, feet, -1.0, 54.0);
            }
            self.sfx.push("super");
            let nm = f.def.display_name.to_uppercase();
            self.tag(x, y - 46.0, &format!("SUPER · {nm}"), gold);
        }
        self.flash_add(0.25);
        self.chroma = self.chroma.max(0.55);
        self.radial = 0.5;
        self.trauma = (self.trauma + 0.2).min(0.7);
        self.lines = 0.8;
        self.lines_col = [1.0, 0.85, 0.4];
    }

    pub(super) fn on_transform(&mut self, fighter: u32, m: &Match) {
        self.super_t = 1.0;
        self.super_owner = fighter;
        let mut col = [1.0, 0.85, 0.4];
        if let Some(f) = m.fighters.iter().find(|f| f.id == fighter) {
            col = Self::form_color(f);
            let (x, y, _) = Self::chest_of(f);
            let feet = f.pos.y + f.lane;
            for (i, r) in [70.0f32, 140.0, 210.0].iter().enumerate() {
                self.ring(x, y, *r, 0.55 + i as f32 * 0.2, 2.8 - i as f32 * 0.7, col);
            }
            self.ring(f.pos.x, feet - 1.0, 190.0, 0.9, 2.4, [1.0, 1.0, 1.0]);
            self.glow(x, y, 80.0, 0.8, col);
            self.glow(x, y, 30.0, 0.5, [1.0, 1.0, 1.0]);
            self.light(x, y, 240.0, 1.1, col);
            self.pillar(f.pos.x, feet, col, quality::count(34));
            self.spark_burst(
                x,
                y,
                1.0,
                quality::count(34),
                10.0,
                col,
                std::f32::consts::PI,
            );
            self.debris(f.pos.x, feet, quality::count(10), [0.7, 0.65, 0.6]);
            self.dust(
                f.pos.x,
                feet,
                quality::count(14),
                3.2,
                1.0,
                [0.85, 0.8, 0.75],
            );
            if quality::level() >= 2 {
                self.cracks(f.pos.x, feet, 1.0, 64.0);
                self.cracks(f.pos.x, feet, -1.0, 64.0);
            }
            let nm = f.active_form_name().to_uppercase();
            self.tag(
                x,
                y - 46.0,
                if nm.is_empty() { crate::i18n::tr3("DESPERTAR", "AWAKEN", "DESPERTAR") } else { &nm },
                col,
            );
            if f.def.boss && f.form >= 3 {
                self.banner(crate::i18n::tr3("ECLIPSE PRIMORDIAL", "PRIMORDIAL ECLIPSE", "ECLIPSE PRIMORDIAL"), crate::i18n::tr3("FORMA SUPREMA DE MIKAZUKI OBORO", "SUPREME FORM OF MIKAZUKI OBORO", "FORMA SUPREMA DE MIKAZUKI OBORO"), col);
            } else if f.def.boss && f.form >= 2 {
                self.banner(crate::i18n::tr3("ECLIPSE ABSOLUTO", "ABSOLUTE ECLIPSE", "ECLIPSE ABSOLUTO"), crate::i18n::tr3("FORMA FINAL DE MIKAZUKI OBORO", "FINAL FORM OF MIKAZUKI OBORO", "FORMA FINAL DE MIKAZUKI OBORO"), col);
            }
        }
        self.flash_add(0.4);
        self.chroma = 0.7;
        self.radial = 0.6;
        self.trauma = 0.6;
        self.vig = 0.3;
        self.lines = 0.8;
        self.lines_col = col;
        self.sfx.push("transform");
    }

    fn pillar(&mut self, x: f32, feet: f32, col: [f32; 3], n: usize) {
        for _ in 0..n {
            let px = x + self.rr(-16.0, 16.0);
            let life = self.rr(0.7, 1.5);
            let vy = -self.rr(1.6, 5.0);
            let size = self.rr(1.0, 2.2);
            let c = if self.r() > 0.35 {
                col
            } else {
                [1.0, 1.0, 1.0]
            };
            let vx = self.rr(-0.25, 0.25);
            self.ember(px, feet, vx, vy, -0.012, life, size, c);
        }
        let rot = self.rr(0.0, TAU);
        self.parts.push(P {
            k: Kind::Glow,
            x,
            y: feet - 40.0,
            vx: 0.0,
            vy: -0.3,
            g: 0.0,
            life: 0.6,
            max: 0.6,
            size: 26.0,
            col,
            rot,
            vrot: 0.0,
        });
    }
}
