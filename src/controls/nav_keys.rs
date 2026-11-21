use crate::gfx::KeyCode;

pub(super) const UP: &[KeyCode] = &[KeyCode::Up, KeyCode::W];
pub(super) const DOWN: &[KeyCode] = &[KeyCode::Down, KeyCode::S];
pub(super) const LEFT: &[KeyCode] = &[KeyCode::Left, KeyCode::A];
pub(super) const RIGHT: &[KeyCode] = &[KeyCode::Right, KeyCode::D];
pub(super) const OK: &[KeyCode] = &[KeyCode::Enter, KeyCode::Space, KeyCode::Q, KeyCode::Kp1];
pub(super) const BACK: &[KeyCode] = &[KeyCode::Escape, KeyCode::Backspace];
pub(super) const AUX: &[KeyCode] = &[KeyCode::F2];

#[cfg(test)]
mod tests {
    use super::*;

    const ACTIONS: [&[KeyCode]; 7] = [UP, DOWN, LEFT, RIGHT, OK, BACK, AUX];

    #[test]
    fn menu_actions_never_share_a_key() {
        for (i, action) in ACTIONS.iter().enumerate() {
            for other in &ACTIONS[i + 1..] {
                assert!(
                    action.iter().all(|key| !other.contains(key)),
                    "a menu key drives two actions"
                );
            }
        }
    }

    #[test]
    fn w_is_menu_up_and_never_menu_back() {
        assert!(UP.contains(&KeyCode::W));
        assert!(!BACK.contains(&KeyCode::W));
    }
}
