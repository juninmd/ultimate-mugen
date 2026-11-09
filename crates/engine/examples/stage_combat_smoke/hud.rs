pub fn draw_rect(rgba: &mut [u8], w: usize, h: usize, x0: usize, y0: usize, rw: usize, rh: usize, color: [u8; 4]) {
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

pub fn draw_hud(rgba: &mut [u8], w: usize, h: usize, stage_name: &str, life1_ratio: f32, life2_ratio: f32, hits: i32) {
    let bar_w = (w as f32 * 0.35) as usize;
    let bar_h = 16;
    let y = 24;

    // P1 Life Bar (Left)
    draw_rect(rgba, w, h, 20, y, bar_w, bar_h, [30, 30, 40, 200]);
    draw_rect(rgba, w, h, 18, y - 2, bar_w + 4, 2, [200, 200, 220, 255]);
    draw_rect(rgba, w, h, 18, y + bar_h, bar_w + 4, 2, [200, 200, 220, 255]);
    let fill1 = ((bar_w - 4) as f32 * life1_ratio.clamp(0.0, 1.0)) as usize;
    if fill1 > 0 {
        let col = if life1_ratio > 0.3 { [60, 220, 120, 255] } else { [230, 60, 50, 255] };
        draw_rect(rgba, w, h, 22 + (bar_w - 4 - fill1), y + 2, fill1, bar_h - 4, col);
    }

    // P2 Life Bar (Right)
    let p2_x = w - 20 - bar_w;
    draw_rect(rgba, w, h, p2_x, y, bar_w, bar_h, [30, 30, 40, 200]);
    draw_rect(rgba, w, h, p2_x - 2, y - 2, bar_w + 4, 2, [200, 200, 220, 255]);
    draw_rect(rgba, w, h, p2_x - 2, y + bar_h, bar_w + 4, 2, [200, 200, 220, 255]);
    let fill2 = ((bar_w - 4) as f32 * life2_ratio.clamp(0.0, 1.0)) as usize;
    if fill2 > 0 {
        let col = if life2_ratio > 0.3 { [240, 180, 40, 255] } else { [230, 60, 50, 255] };
        draw_rect(rgba, w, h, p2_x + 2, y + 2, fill2, bar_h - 4, col);
    }

    // Timer Box (Center)
    let box_sz = 34;
    let t_x = (w - box_sz) / 2;
    draw_rect(rgba, w, h, t_x, y - 9, box_sz, box_sz, [20, 24, 32, 240]);
    draw_rect(rgba, w, h, t_x - 1, y - 10, box_sz + 2, 2, [255, 215, 0, 255]);
    draw_rect(rgba, w, h, t_x - 1, y + box_sz - 9, box_sz + 2, 2, [255, 215, 0, 255]);
    draw_rect(rgba, w, h, t_x + 7, y - 2, 8, 20, [255, 230, 90, 255]);
    draw_rect(rgba, w, h, t_x + 19, y - 2, 8, 20, [255, 230, 90, 255]);

    // Bottom Stage Title Banner
    let banner_w = (w as f32 * 0.70) as usize;
    let banner_h = 24;
    let bx = (w - banner_w) / 2;
    let by = h - banner_h - 12;
    draw_rect(rgba, w, h, bx, by, banner_w, banner_h, [10, 14, 24, 220]);
    draw_rect(rgba, w, h, bx, by, banner_w, 2, [80, 160, 255, 255]);
    draw_rect(rgba, w, h, bx, by + banner_h - 2, banner_w, 2, [80, 160, 255, 255]);

    // Hit counter indicator if hits landed
    if hits > 0 {
        draw_rect(rgba, w, h, 20, y + bar_h + 8, 90, 18, [220, 40, 40, 220]);
        draw_rect(rgba, w, h, 20, y + bar_h + 8, 90, 2, [255, 255, 100, 255]);
    }
    let _ = stage_name;
}
