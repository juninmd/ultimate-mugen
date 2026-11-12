use super::Fighter;

impl Fighter {
    pub fn sprite_scale(&self) -> [f32; 2] {
        if self.size_overrides.is_empty() {
            return self.draw_scale;
        }
        let rx = self.c("size.xscale", 1.0) / self.anim_def.scale_x.max(f32::EPSILON);
        let ry = self.c("size.yscale", 1.0) / self.anim_def.scale_y.max(f32::EPSILON);
        [self.draw_scale[0] * rx, self.draw_scale[1] * ry]
    }
}
