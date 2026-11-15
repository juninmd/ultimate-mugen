use super::*;

impl StageReact {
    pub fn new(kind: StageKind) -> Self {
        StageReact {
            kind,
            look: look(kind),
            t: 0.0,
            rng: 0x1234_5679,
            explicit: false,
            queue: vec![],
            shake: 0.0,
            flicker: 0.0,
            gust: 0.0,
            cheer: 0.0,
            swing: 0.0,
            swing_v: 0.0,
            rotor: 0.0,
            shocks: vec![],
            decals: vec![],
            ripples: vec![],
            bits: vec![],
            prev: vec![],
            round_no: -1,
            was_super: false,
            applied: 0,
            #[cfg(not(target_arch = "wasm32"))]
            off: std::env::var("UME_NOFX").is_ok(),
            #[cfg(target_arch = "wasm32")]
            off: false,
        }
    }
    pub fn time(&self) -> f32 {
        self.t
    }
    pub fn active(&self) -> bool {
        self.kind != StageKind::Custom && !self.off
    }
    /// Queue explicit impulses (switches off the automatic detection from the `Match`).
    pub fn react(&mut self, imps: &[Impulse]) {
        self.explicit = true;
        self.queue.extend_from_slice(imps);
    }
    /// Reflection strength (0 = matte) and horizontal ripple wobble of the floor.
    pub fn gloss(&self) -> (f32, f32) {
        (self.look.gloss, self.look.wobble)
    }
    /// Multiplicative lighting colour for a fighter standing at world x (warm sunset, neon side lights, lava flicker...).
    pub fn ambient(&self, x: f32, cam_x: f32) -> Color {
        let t = self.look.tint;
        let side = ((x - cam_x) / 200.0).clamp(-1.0, 1.0);
        let mut c = [t[0], t[1], t[2]];
        match self.kind {
            StageKind::Neon => {
                c = [
                    t[0] + side * 0.1,
                    t[1] - side.abs() * 0.04,
                    t[2] - side * 0.1,
                ];
            }
            StageKind::Volcano => {
                let f = 0.5 + 0.5 * (self.t * 7.0).sin() * (0.5 + self.flicker);
                c[1] += f * 0.07;
                c[2] += f * 0.05;
            }
            _ => {}
        }
        Color::new(c[0].min(1.0), c[1].min(1.0), c[2].min(1.0), 1.0)
    }
    pub(super) fn r(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x as f32 / 4294967296.0
    }
    pub(super) fn rr(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.r()
    }
}
