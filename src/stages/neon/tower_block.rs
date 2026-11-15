use super::*;

pub(super) fn tower(
    pm: &mut Pixmap,
    t: Transform,
    x: f32,
    w: f32,
    h: f32,
    base: f32,
    body: &str,
    seed: u32,
    win_p: f32,
) {
    rect(
        pm,
        t,
        x,
        base - h,
        w,
        h + 40.0,
        &lin(base - h, base + 30.0, &[(0.0, body), (1.0, "#05030c")]),
    );
    let mut y = base - h + 4.0;
    let mut row = 0u32;
    while y < base {
        let mut xx = x + 2.0;
        let mut col = 0u32;
        while xx < x + w - 2.0 {
            if hash(row * 31 + col, seed) < win_p {
                let c = if hash(row, col + seed) < 0.55 {
                    rgba("#8fd4ff", 0.55)
                } else if hash(col, row + seed) < 0.5 {
                    rgba("#ff9ae0", 0.5)
                } else {
                    rgba("#ffe4a0", 0.45)
                };
                rect(pm, t, xx, y, 1.8, 2.2, &solid(c));
            }
            xx += 3.8;
            col += 1;
        }
        y += 4.6;
        row += 1;
    }
}
