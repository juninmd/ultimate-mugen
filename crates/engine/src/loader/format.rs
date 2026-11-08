use ume_formats::ini::parse_float;
/// `localcoord` -> (largura, altura). Aceita o formato canonico `320,240`,
/// o valor unico usado por varios chars de fullgames (`400` vira `400x240`)
/// e o campo ausente/invalido (320x240).
pub(super) fn local_size(lc: Option<&str>) -> (f32, f32) {
    let Some(lc) = lc else {
        return (320.0, 240.0);
    };
    let lc = lc.trim();
    if lc.is_empty() {
        return (320.0, 240.0);
    };
    if let Some((w_s, h_s)) = lc.split_once(',') {
        let w = parse_float(w_s)
            .map(|v| v as f32)
            .filter(|w| *w > 0.0)
            .unwrap_or(320.0);
        let h = parse_float(h_s)
            .map(|v| v as f32)
            .filter(|h| *h > 0.0)
            .unwrap_or(240.0);
        (w, h)
    } else {
        let w = parse_float(lc)
            .map(|v| v as f32)
            .filter(|w| *w > 0.0)
            .unwrap_or(320.0);
        (w, 240.0)
    }
}
