pub fn draw_rect(
    rgba: &mut [u8],
    w: usize,
    h: usize,
    x0: usize,
    y0: usize,
    rw: usize,
    rh: usize,
    color: [u8; 4],
) {
    let (x1, y1) = ((x0 + rw).min(w), (y0 + rh).min(h));
    for y in y0..y1 {
        for x in x0..x1 {
            let idx = (y * w + x) * 4;
            if color[3] == 255 {
                rgba[idx..idx + 4].copy_from_slice(&color);
            } else {
                let a = color[3] as f32 / 255.0;
                let inv = 1.0 - a;
                rgba[idx] = (color[0] as f32 * a + rgba[idx] as f32 * inv) as u8;
                rgba[idx + 1] = (color[1] as f32 * a + rgba[idx + 1] as f32 * inv) as u8;
                rgba[idx + 2] = (color[2] as f32 * a + rgba[idx + 2] as f32 * inv) as u8;
                rgba[idx + 3] = 255;
            }
        }
    }
}

fn draw_lifebar(
    rgba: &mut [u8],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    bar_w: usize,
    bar_h: usize,
    ratio: f32,
    is_p1: bool,
) {
    let bg_color = [30, 30, 40, 200];
    let border_color = [200, 200, 220, 255];
    let fill_color = if ratio > 0.3 {
        if is_p1 {
            [60, 220, 120, 255]
        } else {
            [240, 180, 40, 255]
        }
    } else {
        [230, 60, 50, 255]
    };
    let bx = x.saturating_sub(2);
    let by_top = y.saturating_sub(2);
    let by_bot = y + bar_h;
    let full_w = bar_w + 4;

    draw_rect(rgba, w, h, x, y, bar_w, bar_h, bg_color);
    draw_rect(rgba, w, h, bx, by_top, full_w, 2, border_color);
    draw_rect(rgba, w, h, bx, by_bot, full_w, 2, border_color);

    let inner_w = bar_w.saturating_sub(4);
    let fill = ((inner_w as f32) * ratio.clamp(0.0, 1.0)) as usize;
    if fill > 0 {
        let fill_x = if is_p1 {
            x + 2 + (inner_w.saturating_sub(fill))
        } else {
            x + 2
        };
        let fill_h = bar_h.saturating_sub(4);
        draw_rect(rgba, w, h, fill_x, y + 2, fill, fill_h, fill_color);
    }
}

pub fn draw_hud(
    rgba: &mut [u8],
    w: usize,
    h: usize,
    stage_name: &str,
    life1_ratio: f32,
    life2_ratio: f32,
    hits: i32,
) {
    let bar_w = (w as f32 * 0.35) as usize;
    let bar_h = 16;
    let y = 24;

    // P1 & P2 Life Bars
    draw_lifebar(rgba, w, h, 20, y, bar_w, bar_h, life1_ratio, true);
    let p2_x = w.saturating_sub(20 + bar_w);
    draw_lifebar(rgba, w, h, p2_x, y, bar_w, bar_h, life2_ratio, false);

    // Timer Box (Center)
    let box_sz = 34;
    let t_x = w.saturating_sub(box_sz) / 2;
    let t_top = y.saturating_sub(9);
    draw_rect(rgba, w, h, t_x, t_top, box_sz, box_sz, [20, 24, 32, 240]);
    draw_rect(
        rgba,
        w,
        h,
        t_x.saturating_sub(1),
        y.saturating_sub(10),
        box_sz + 2,
        2,
        [255, 215, 0, 255],
    );
    draw_rect(
        rgba,
        w,
        h,
        t_x.saturating_sub(1),
        y + box_sz.saturating_sub(9),
        box_sz + 2,
        2,
        [255, 215, 0, 255],
    );
    draw_rect(
        rgba,
        w,
        h,
        t_x + 7,
        y.saturating_sub(2),
        8,
        20,
        [255, 230, 90, 255],
    );
    draw_rect(
        rgba,
        w,
        h,
        t_x + 19,
        y.saturating_sub(2),
        8,
        20,
        [255, 230, 90, 255],
    );

    // Bottom Stage Title Banner
    let banner_w = (w as f32 * 0.70) as usize;
    let banner_h = 24;
    let bx = w.saturating_sub(banner_w) / 2;
    let by = h.saturating_sub(banner_h + 12);
    draw_rect(rgba, w, h, bx, by, banner_w, banner_h, [10, 14, 24, 220]);
    draw_rect(rgba, w, h, bx, by, banner_w, 2, [80, 160, 255, 255]);
    draw_rect(
        rgba,
        w,
        h,
        bx,
        by + banner_h.saturating_sub(2),
        banner_w,
        2,
        [80, 160, 255, 255],
    );

    // Hit counter indicator if hits landed
    if hits > 0 {
        let hit_y = y + bar_h + 8;
        draw_rect(rgba, w, h, 20, hit_y, 90, 18, [220, 40, 40, 220]);
        draw_rect(rgba, w, h, 20, hit_y, 90, 2, [255, 255, 100, 255]);
    }
    let _ = stage_name;
}
