pub const STRIPS: usize = 28;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CardState {
    Pending,
    Loading,
    Ready,
}

pub fn card_state(index: usize, step: usize) -> CardState {
    match index.cmp(&step) {
        std::cmp::Ordering::Less => CardState::Ready,
        std::cmp::Ordering::Equal => CardState::Loading,
        std::cmp::Ordering::Greater => CardState::Pending,
    }
}

pub fn strip_alpha(reveal: f32, strip: usize) -> f32 {
    let row = strip as f32 / STRIPS as f32;
    ((reveal * 1.25 - row) * 5.0).clamp(0.0, 1.0)
}

pub fn edge_fraction(reveal: f32) -> f32 {
    (reveal * 1.25).clamp(0.0, 1.0)
}

pub fn card_reveal(state: CardState, time: f32) -> f32 {
    match state {
        CardState::Pending => 0.0,
        CardState::Loading => 0.45 + 0.35 * (0.5 + 0.5 * (time * 2.2).sin()),
        CardState::Ready => 1.0,
    }
}

pub fn jitter(strip: usize, frame: u32) -> f32 {
    let h = (strip as u32).wrapping_mul(2654435761) ^ frame.wrapping_mul(40503);
    ((h >> 7) & 0xff) as f32 / 255.0 - 0.5
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mosaic {
    pub cols: usize,
    pub w: f32,
    pub h: f32,
    pub labels: bool,
}

pub const LABEL_H: f32 = 44.0;
const LABEL_MIN_W: f32 = 110.0;
const CARD_ASPECT: f32 = 1.25;
const GAP: f32 = 12.0;

pub fn mosaic(n: usize, area_w: f32, area_h: f32, max_w: f32) -> Mosaic {
    let n = n.max(1);
    let fit = |labels: bool| {
        let extra = if labels { LABEL_H } else { 0.0 };
        let mut best = Mosaic { cols: 1, w: 1.0, h: CARD_ASPECT, labels };
        for cols in 1..=n {
            let rows = n.div_ceil(cols) as f32;
            let by_w = (area_w - (cols - 1) as f32 * GAP) / cols as f32;
            let by_h = ((area_h - (rows - 1.0) * GAP) / rows - extra) / CARD_ASPECT;
            let w = by_w.min(by_h).min(max_w);
            if w > best.w {
                best = Mosaic { cols, w, h: w * CARD_ASPECT, labels };
            }
        }
        best
    };
    let with_labels = fit(true);
    if with_labels.w >= LABEL_MIN_W {
        with_labels
    } else {
        fit(false)
    }
}

pub fn cell_height(m: &Mosaic) -> f32 {
    m.h + if m.labels { LABEL_H } else { 0.0 }
}

pub fn origin(m: &Mosaic, index: usize, n: usize, area_x: f32, area_y: f32, area_w: f32, area_h: f32) -> (f32, f32) {
    let rows = n.div_ceil(m.cols);
    let (row, col) = (index / m.cols, index % m.cols);
    let in_row = if row + 1 == rows { n - row * m.cols } else { m.cols };
    let row_w = in_row as f32 * m.w + (in_row - 1) as f32 * GAP;
    let total_h = rows as f32 * cell_height(m) + (rows - 1) as f32 * GAP;
    (
        area_x + (area_w - row_w) / 2.0 + col as f32 * (m.w + GAP),
        area_y + (area_h - total_h) / 2.0 + row as f32 * (cell_height(m) + GAP),
    )
}

pub fn revealed(step: usize, total: usize) -> usize {
    (step + 3).min(total).max(2.min(total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_follow_the_loading_step() {
        assert_eq!(card_state(0, 1), CardState::Ready);
        assert_eq!(card_state(1, 1), CardState::Loading);
        assert_eq!(card_state(2, 1), CardState::Pending);
    }

    #[test]
    fn strips_appear_top_down_and_finish_solid() {
        assert_eq!(strip_alpha(0.0, 0), 0.0);
        assert!(strip_alpha(0.4, 2) > strip_alpha(0.4, 20));
        assert!((0..STRIPS).all(|s| strip_alpha(1.0, s) == 1.0));
    }

    #[test]
    fn loading_reveal_stays_partial() {
        assert!((0..200).all(|t| (0.4..0.9).contains(&card_reveal(CardState::Loading, t as f32 * 0.07))));
    }

    #[test]
    fn mosaic_fits_any_roster_inside_the_area() {
        for n in [1, 2, 4, 16, 64, 120, 400] {
            let m = mosaic(n, 1160.0, 330.0, 190.0);
            let rows = n.div_ceil(m.cols) as f32;
            assert!(m.cols as f32 * m.w + (m.cols - 1) as f32 * 12.0 <= 1160.5, "{n}");
            assert!(rows * cell_height(&m) + (rows - 1.0) * 12.0 <= 330.5, "{n}");
            assert!(m.w >= 1.0 && m.w <= 190.0);
        }
    }

    #[test]
    fn mosaic_shrinks_as_the_roster_grows() {
        let sizes: Vec<f32> = [2, 12, 60, 150].iter().map(|&n| mosaic(n, 1160.0, 330.0, 190.0).w).collect();
        assert!(sizes.windows(2).all(|w| w[0] >= w[1]));
        assert!(mosaic(2, 1160.0, 330.0, 190.0).labels && !mosaic(150, 1160.0, 330.0, 190.0).labels);
    }

    #[test]
    fn reveal_runs_ahead_of_the_step_and_caps_at_total() {
        assert_eq!(revealed(0, 120), 3);
        assert_eq!(revealed(200, 120), 120);
        assert_eq!(revealed(0, 1), 1);
    }

    #[test]
    fn jitter_is_bounded() {
        assert!((0..STRIPS).all(|s| (0..50).all(|f| jitter(s, f).abs() <= 0.5)));
    }
}
