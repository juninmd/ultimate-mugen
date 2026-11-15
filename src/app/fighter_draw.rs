use super::*;

impl App {
    pub fn draw_char(
        &mut self,
        d: &Arc<CharDef>,
        anim: i32,
        t: f32,
        x: f32,
        y: f32,
        height_px: f32,
        mirror: bool,
    ) {
        let base_h = (d.c("size.height", 60.0) as f32 * 1.7).max(90.0);
        self.draw_char_k(d, anim, t, x, y, height_px / base_h, mirror);
    }

    pub(super) fn draw_char_k(
        &mut self,
        d: &Arc<CharDef>,
        anim: i32,
        t: f32,
        x: f32,
        y: f32,
        kv: f32,
        mirror: bool,
    ) {
        let (form_g, anim) = if anim <= -2000 {
            (4, -anim - 2000)
        } else if anim <= -1000 {
            (3, -anim - 1000)
        } else {
            (1, anim)
        };
        let Some(a) = d.anims.get(&anim).or(d.anims.get(&0)) else {
            return;
        };
        if a.frames.is_empty() {
            return;
        }
        let tot = a.total_time.max(1);
        let tm = ((t * 60.0) as i32) % tot;
        let (mut acc, mut idx) = (0, 0);
        for (i, f) in a.frames.iter().enumerate() {
            idx = i;
            if f.time < 0 || tm < acc + f.time {
                break;
            }
            acc += f.time;
        }
        let fr = &a.frames[idx];
        let key = if form_g > 1
            && fr.group == 1
            && d.sprites.contains_key(&(form_g, fr.index))
        {
            (form_g, fr.index)
        } else {
            (fr.group, fr.index)
        };
        let Some(spr) = d.sprites.get(&key) else { return };
        let Some(tex) = self.g.sprite_tex(d, key, &[]) else { return };
        let l = 320.0 / d.local_w;
        let k = kv * self.ui.s;
        let ks = k * spr.scale * l;
        let flip = mirror ^ fr.flip_h;
        let w = spr.width * ks;
        let h = spr.height * ks;
        let px = self.ui.x(x) + fr.x * l * k * if mirror { -1.0 } else { 1.0 };
        let py = self.ui.y(y) + fr.y * l * k;
        let left = if flip { px - (spr.width - spr.axis_x) * ks } else { px - spr.axis_x * ks };
        tex.draw(left, py - spr.axis_y * ks, self.char_tint, DrawTextureParams { dest_size: Some(vec2(w, h)), flip_x: flip, ..Default::default() });
    }

    pub(super) fn char_extent(&mut self, d: &Arc<CharDef>) -> [f32; 4] {
        if let Some(e) = self.fit.get(d.id.as_str()) {
            return *e;
        }
        let l = 320.0 / d.local_w;
        let mut e = [f32::MAX, f32::MIN, f32::MAX, f32::MIN];
        if let Some(a) = d.anims.get(&0) {
            for fr in &a.frames {
                let Some(s) = d.sprites.get(&(fr.group, fr.index)) else {
                    continue;
                };
                let sc = s.scale * l;
                let (ox, oy, ow, oh) = opaque_box(d, (fr.group, fr.index), s);
                let (top, left) = (fr.y * l + (oy - s.axis_y) * sc, if fr.flip_h { fr.x * l + (s.width - ow - ox - s.axis_x) * sc } else { fr.x * l + (ox - s.axis_x) * sc });
                e = [e[0].min(top), e[1].max(top + oh * sc), e[2].min(left), e[3].max(left + ow * sc)];
            }
        }
        if e[0] >= e[1] || e[2] >= e[3] {
            e = [-150.0, 0.0, -60.0, 60.0];
        }
        self.fit.insert(d.id.clone(), e);
        e
    }

    pub fn draw_char_fit(
        &mut self,
        d: &Arc<CharDef>,
        anim: i32,
        t: f32,
        cx: f32,
        bottom: f32,
        fw: f32,
        fh: f32,
        mirror: bool,
    ) {
        let e = self.char_extent(d);
        let k = (fh / (e[1] - e[0])).min(fw / (e[3] - e[2])).min(6.0);
        let sgn = if mirror { -1.0 } else { 1.0 };
        self.draw_char_k(
            d,
            anim,
            t,
            cx - sgn * (e[2] + e[3]) * 0.5 * k,
            bottom - e[1] * k,
            k,
            mirror,
        );
    }
}
