fn parse_channel(text: &str, start: usize) -> Option<f64> {
    u8::from_str_radix(text.get(start..start + 2)?, 16).ok().map(|v| v as f64 / 255.0)
}

fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let high = r.max(g).max(b);
    let low = r.min(g).min(b);
    let span = high - low;
    if span == 0.0 {
        return (0.0, 0.0, high);
    }
    let saturation = span / high;
    let (rc, gc, bc) = ((high - r) / span, (high - g) / span, (high - b) / span);
    let hue = if r == high { bc - gc } else if g == high { 2.0 + rc - bc } else { 4.0 + gc - rc };
    ((hue / 6.0).rem_euclid(1.0), saturation, high)
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    if s == 0.0 {
        return (v, v, v);
    }
    let sector = (h * 6.0).floor();
    let fraction = h * 6.0 - sector;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * fraction), v * (1.0 - s * (1.0 - fraction)));
    match (sector as i64).rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

pub fn shift(color: &str, degrees: f64, sat: f64, val: f64) -> String {
    if !(color.starts_with('#') && color.len() == 7) {
        return color.to_string();
    }
    let channels = (parse_channel(color, 1), parse_channel(color, 3), parse_channel(color, 5));
    let (Some(r), Some(g), Some(b)) = channels else { return color.to_string() };
    let (h, s, v) = rgb_to_hsv(r, g, b);
    let (r, g, b) = hsv_to_rgb((h + degrees / 360.0).rem_euclid(1.0), (s * sat).min(1.0), (v * val).min(1.0));
    let byte = |x: f64| (x * 255.0).round_ties_even() as u8;
    format!("#{:02x}{:02x}{:02x}", byte(r), byte(g), byte(b))
}
