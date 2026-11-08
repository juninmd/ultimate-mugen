/// True when a portrait has enough colour range to be more than a blank placeholder: at least one
/// reasonably bright pixel, or a visible average above near-black.
pub fn portrait_pixels_usable(rgba: &[u8]) -> bool {
    let (mut visible, mut sum, mut max) = (0u64, 0u64, 0u8);
    for p in rgba.chunks_exact(4).filter(|p| p[3] > 16) {
        let l = ((p[0] as u16 + p[1] as u16 + p[2] as u16) / 3) as u8;
        visible += 1;
        sum += l as u64;
        max = max.max(l);
    }
    visible > 0 && (max >= 16 || sum as f64 / visible as f64 >= 8.0)
}

#[cfg(test)]
mod portrait_tests {
    use super::portrait_pixels_usable;

    #[test]
    fn transparent_portraits_are_rejected() {
        assert!(!portrait_pixels_usable(&[0u8; 256]));
    }

    #[test]
    fn solid_dark_placeholders_are_rejected() {
        let mut rgba = Vec::new();
        for _ in 0..64 { rgba.extend_from_slice(&[2, 2, 3, 255]); }
        assert!(!portrait_pixels_usable(&rgba));
    }

    #[test]
    fn portraits_with_visible_colour_are_kept() {
        let mut rgba = vec![0u8; 64 * 4];
        rgba[40..44].copy_from_slice(&[180, 120, 90, 255]);
        assert!(portrait_pixels_usable(&rgba));
    }

    #[test]
    fn uniformly_dark_but_visible_portraits_are_kept() {
        let mut rgba = Vec::new();
        for _ in 0..64 { rgba.extend_from_slice(&[30, 20, 25, 255]); }
        assert!(portrait_pixels_usable(&rgba));
    }
}
