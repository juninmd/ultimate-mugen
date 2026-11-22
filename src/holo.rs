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
    fn jitter_is_bounded() {
        assert!((0..STRIPS).all(|s| (0..50).all(|f| jitter(s, f).abs() <= 0.5)));
    }
}
