use super::quality;
use super::sheet::SheetKind;
use super::{Kind, Vfx, P};

impl Vfx {
    pub(super) fn dash_burst(&mut self, x: f32, feet: f32, facing: f32) {
        let behind = if facing >= 0.0 {
            std::f32::consts::PI
        } else {
            0.0
        };
        self.sheets.spawn_facing(SheetKind::DashDust, x - facing * 6.0, feet, 54.0, facing);
        for _ in 0..quality::count(4) {
            let height = self.rr(8.0, 52.0);
            let size = self.rr(26.0, 52.0);
            let offset = self.rr(-6.0, 10.0);
            self.parts.push(P {
                k: Kind::Slash,
                x: x - facing * offset,
                y: feet - height,
                vx: 0.0,
                vy: 0.0,
                g: 0.0,
                life: 0.16,
                max: 0.16,
                size,
                col: [0.85, 0.92, 1.0],
                rot: behind,
                vrot: 0.0,
            });
        }
        self.dust(
            x - facing * 8.0,
            feet,
            quality::count(5),
            1.4,
            0.3,
            [0.85, 0.82, 0.78],
        );
        if quality::level() >= 1 {
            self.ring(x, feet - 1.0, 22.0, 0.2, 1.0, [0.9, 0.9, 0.95]);
        }
    }
}
