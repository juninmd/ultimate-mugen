use super::*;

impl<'a> Ctx<'a> {
    pub fn new(g: &Gfx, m: &Match, layers: &'a [Layer], shx: f32, shy: f32) -> Self {
        Ctx {
            s: g.view_scale,
            ox: g.ox,
            oy: g.oy,
            cam_x: m.camera.x,
            cam_y: m.camera.y,
            vw: m.view_w,
            vh: m.view_h,
            zoff: m.stage.zoffset,
            shx,
            shy,
            layers,
        }
    }
    /// World point (floor line = y 0, up is negative) → screen.
    pub fn w(&self, x: f32, y: f32) -> Vec2 {
        vec2(
            self.ox + (x - self.cam_x + self.vw / 2.0 + self.shx) * self.s,
            self.oy + (y - self.cam_y + self.zoff + self.shy) * self.s,
        )
    }
    /// Point in the local coordinates of back layer `i`.
    pub fn l(&self, i: usize, x: f32, y: f32) -> Vec2 {
        let l = &self.layers[i];
        let (lx, ly) = (
            self.vw / 2.0 - l.w / 2.0 - self.cam_x * l.delta + self.shx,
            l.y - self.cam_y * l.dy + (self.zoff - 205.0) + self.shy,
        );
        vec2(self.ox + (lx + x) * self.s, self.oy + (ly + y) * self.s)
    }
    /// World x that a layer-local x appears above (to look up shock waves / impacts).
    pub fn lw(&self, i: usize, x: f32) -> f32 {
        let l = &self.layers[i];
        self.vw / 2.0 - l.w / 2.0 - self.cam_x * l.delta + x - self.vw / 2.0 + self.cam_x
    }
    /// Screen-fixed view coordinates with a parallax factor (foreground sprinkles).
    pub fn v(&self, x: f32, y: f32) -> Vec2 {
        vec2(
            self.ox + (x + self.shx) * self.s,
            self.oy + (y + self.shy) * self.s,
        )
    }
    pub fn view(&self) -> (f32, f32) {
        (self.vw, self.vh)
    }
    pub fn cam(&self) -> f32 {
        self.cam_x
    }
}
