use super::quality;
use super::{Kind, Vfx, P};
use ume_engine::matchs::Match;

struct Tier {
    sparks: usize,
    speed: f32,
    glow: f32,
    ring: f32,
    trauma: f32,
    punch: f32,
    flash: f32,
    debris: usize,
    crack: f32,
    slashes: usize,
    slash_len: f32,
    flare: f32,
    col: [f32; 3],
    core: [f32; 3],
    chroma: f32,
}

fn tier(damage: f32, heavy: bool) -> Tier {
    if damage >= 150.0 {
        Tier {
            sparks: 26,
            speed: 9.5,
            glow: 30.0,
            ring: 58.0,
            trauma: 0.55,
            punch: 0.034,
            flash: 0.22,
            debris: 9,
            crack: 46.0,
            slashes: 6,
            slash_len: 40.0,
            flare: 34.0,
            col: [1.0, 0.36, 0.12],
            core: [1.0, 0.95, 0.8],
            chroma: 0.35,
        }
    } else if heavy || damage >= 90.0 {
        Tier {
            sparks: 17,
            speed: 7.5,
            glow: 22.0,
            ring: 40.0,
            trauma: 0.4,
            punch: 0.024,
            flash: 0.1,
            debris: 5,
            crack: 30.0,
            slashes: 4,
            slash_len: 30.0,
            flare: 24.0,
            col: [1.0, 0.5, 0.14],
            core: [1.0, 0.95, 0.75],
            chroma: 0.15,
        }
    } else if damage >= 40.0 {
        Tier {
            sparks: 11,
            speed: 5.5,
            glow: 16.0,
            ring: 28.0,
            trauma: 0.25,
            punch: 0.014,
            flash: 0.0,
            debris: 0,
            crack: 0.0,
            slashes: 3,
            slash_len: 22.0,
            flare: 16.0,
            col: [1.0, 0.65, 0.2],
            core: [1.0, 0.97, 0.85],
            chroma: 0.0,
        }
    } else {
        Tier {
            sparks: 6,
            speed: 4.0,
            glow: 11.0,
            ring: 18.0,
            trauma: 0.14,
            punch: 0.008,
            flash: 0.0,
            debris: 0,
            crack: 0.0,
            slashes: 2,
            slash_len: 16.0,
            flare: 10.0,
            col: [1.0, 0.8, 0.35],
            core: [1.0, 1.0, 0.9],
            chroma: 0.0,
        }
    }
}

impl Vfx {
    pub(super) fn on_hit(
        &mut self,
        damage: f32,
        guarded: bool,
        heavy: bool,
        attacker: usize,
        defender: usize,
        m: &Match,
    ) {
        let Some(df) = m.fighters.get(defender) else {
            return;
        };
        let (dx, dy, dface) = Self::chest_of(df);
        let dir = if m.fighters.get(attacker).is_some_and(|a| a.pos.x <= dx) {
            1.0
        } else {
            -1.0
        };
        let (x, y) = (dx + dface * 7.0, dy);
        if guarded {
            let big = heavy || damage >= 60.0;
            let n = quality::count(if big { 9 } else { 5 });
            self.spark_burst(x, y, dir, n, 3.0, [0.45, 0.75, 1.0], 0.9);
            self.ring(
                x,
                y,
                if big { 24.0 } else { 14.0 },
                0.2,
                1.2,
                [0.5, 0.8, 1.0],
            );
            self.glow(x, y, 10.0, 0.1, [0.5, 0.8, 1.0]);
            self.trauma = (self.trauma + if big { 0.09 } else { 0.05 }).min(1.0);
            self.tag(x, y - 14.0, "GUARD", [0.55, 0.8, 1.0]);
            return;
        }
        let counter = self.pre_mt.get(defender).copied() == Some('A');
        let t = tier(damage, heavy);
        let feet = df.pos.y + df.lane;
        let grounded = df.state_type != 'A';
        self.spark_burst(x, y, dir, quality::count(t.sparks), t.speed, t.col, 1.0);
        self.spark_burst(
            x,
            y,
            dir,
            quality::count(t.sparks / 3 + 1),
            t.speed * 1.4,
            t.core,
            0.4,
        );
        self.glow(x, y, t.glow, 0.12, t.col);
        self.glow(x, y, t.glow * 0.45, 0.09, t.core);
        self.slashes(x, y, dir, quality::count(t.slashes), t.slash_len, t.core);
        self.ring(x, y, t.ring, 0.24, 1.4, t.col);
        self.flare(x, y, t.flare, t.core);
        self.trauma = (self.trauma + t.trauma * 0.5).min(0.7);
        self.punch = self.punch.max(t.punch);
        if t.flash > 0.0 {
            self.flash_add(t.flash);
        }
        if t.chroma > 0.0 {
            self.chroma = self.chroma.max(t.chroma);
        }
        if quality::level() >= 1 && t.debris > 0 {
            self.debris(x, y + 10.0, quality::count(t.debris), [0.8, 0.72, 0.6]);
        }
        if quality::level() >= 2 && t.crack > 0.0 && grounded {
            self.cracks(df.pos.x + dir * 6.0, feet, dir, t.crack);
            self.light(x, y, t.ring * 2.4, 0.3, t.col);
        }
        if !self.first_hit && m.round_state == 2 && !m.training {
            self.first_hit = true;
            self.tag(x, y - 26.0, "FIRST ATTACK", [0.4, 0.9, 1.0]);
        }
        let mag = (damage / 90.0).clamp(0.2, 1.4);
        let col = if counter {
            [1.0, 0.45, 0.35]
        } else {
            [1.0, 0.92, 0.55]
        };
        self.float(
            x + dir * 12.0,
            y - 6.0,
            format!("{}", damage as i32),
            6.0 + mag * 2.5,
            col,
        );
        if counter {
            self.sfx.push("counter");
            self.tag(x, y - 14.0, "COUNTER", [1.0, 0.45, 0.35]);
            self.ring(x, y, 18.0, 0.2, 1.2, [1.0, 0.45, 0.35]);
        }
    }

    fn flare(&mut self, x: f32, y: f32, len: f32, col: [f32; 3]) {
        let arms = if quality::level() == 0 { 2 } else { 4 };
        let spin = self.rr(0.0, 0.8);
        for i in 0..arms {
            let rot = spin + i as f32 * std::f32::consts::FRAC_PI_2;
            self.parts.push(P {
                k: Kind::Slash,
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                g: 0.0,
                life: 0.1,
                max: 0.1,
                size: if i % 2 == 0 { len } else { len * 0.6 },
                col,
                rot,
                vrot: 0.0,
            });
        }
    }

    pub(super) fn cracks(&mut self, x: f32, y: f32, dir: f32, len: f32) {
        for i in 0..3 {
            let side = if i == 1 { -dir } else { dir };
            let rot = side.signum() * self.rr(0.0, 0.25)
                + if side < 0.0 {
                    std::f32::consts::PI
                } else {
                    0.0
                };
            let size = len * self.rr(0.6, 1.1);
            self.parts.push(P {
                k: Kind::Crack,
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                g: 0.0,
                life: 1.4,
                max: 1.4,
                size,
                col: [1.0, 0.55, 0.2],
                rot,
                vrot: 0.0,
            });
        }
    }
}
