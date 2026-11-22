pub(super) const TOP: f32 = 156.0;
pub(super) const VIEW_H: f32 = 492.0;

pub(super) fn offset(rows: usize, focused: Option<usize>, pitch: f32) -> f32 {
    let overflow = (rows as f32 * pitch - (pitch - 54.0) - VIEW_H).max(0.0);
    let Some(row) = focused else { return 0.0 };
    ((row + 1) as f32 * pitch - (pitch - 54.0) - VIEW_H).clamp(0.0, overflow)
}

pub(super) fn visible(y: f32, h: f32) -> bool {
    y >= TOP - 1.0 && y + h <= TOP + VIEW_H + 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_lists_never_scroll() {
        assert_eq!(offset(6, Some(5), 62.0), 0.0);
    }

    #[test]
    fn the_focused_row_stays_inside_the_view() {
        for row in 0..13 {
            let y = TOP + row as f32 * 62.0 - offset(13, Some(row), 62.0);
            assert!(visible(y, 54.0), "row {row} at {y}");
        }
    }

    #[test]
    fn scrolling_stops_at_the_last_row() {
        let last = offset(13, Some(12), 62.0);
        assert!(last > 0.0);
        assert!(TOP + 12.0 * 62.0 - last + 54.0 <= TOP + VIEW_H + 0.01);
    }
}
