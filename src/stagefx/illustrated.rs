use super::*;

impl StageReact {
    pub fn draw_illustrated(&self, g: &Gfx, m: &Match, front: bool, shx: f32, shy: f32) {
        if !self.active() || g.fx_gain <= 0.0 {
            return;
        }
        let cx = Ctx::new(g, m, &[], shx, shy);
        if front {
            self.draw_bits(&cx, g);
        } else {
            self.floor_common(&cx, g, m);
        }
    }
}
