use egui::{pos2, vec2, Color32, Pos2, Rect, Shape, Stroke};

pub fn slab(p: &egui::Painter, x: f32, y: f32, w: f32, h: f32, skew: f32, c: Color32) {
    p.add(Shape::convex_polygon(
        vec![
            pos2(x + skew, y),
            pos2(x + w, y),
            pos2(x + w - skew, y + h),
            pos2(x, y + h),
        ],
        c,
        Stroke::NONE,
    ));
}

pub fn background(p: &egui::Painter, r: Rect, accent: Color32, now: f64) {
    p.rect_filled(r, 0.0, Color32::from_rgb(0x06, 0x14, 0x2a));
    let c = Pos2::new(r.min.x + r.width() * 0.68, r.min.y + r.height() * 0.46);
    let len = r.width() * 1.2;
    for i in 0..14 {
        let a = now as f32 * 0.05 + i as f32 / 14.0 * std::f32::consts::TAU;
        let alpha = if i % 2 == 0 { 26 } else { 13 };
        let col = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), alpha);
        p.add(Shape::convex_polygon(
            vec![
                c,
                c + vec2((a - 0.07).cos(), (a - 0.07).sin()) * len,
                c + vec2((a + 0.07).cos(), (a + 0.07).sin()) * len,
            ],
            col,
            Stroke::NONE,
        ));
    }
}
