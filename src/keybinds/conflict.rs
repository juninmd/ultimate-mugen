use super::{KeyCode, KeyMap};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Conflicts {
    pub swapped_with: Option<usize>,
    pub shared_with: Option<(usize, usize)>,
}

impl KeyMap {
    pub fn conflicts(&self, slot: usize, action: usize, key: KeyCode) -> Conflicts {
        let slot = slot.min(1);
        let swapped_with = self.slots[slot]
            .iter()
            .position(|k| *k == key)
            .filter(|j| *j != action);
        let other = 1 - slot;
        let shared_with = self.slots[other]
            .iter()
            .position(|k| *k == key)
            .map(|j| (other, j));
        Conflicts {
            swapped_with,
            shared_with,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_player_reuse_is_a_swap_and_the_other_player_is_a_shared_key() {
        let m = KeyMap::default();
        let own = m.conflicts(0, 0, m.slots[0][1]);
        assert_eq!(
            own,
            Conflicts {
                swapped_with: Some(1),
                shared_with: None
            }
        );
        let theirs = m.conflicts(0, 0, m.slots[1][3]);
        assert_eq!(
            theirs,
            Conflicts {
                swapped_with: None,
                shared_with: Some((1, 3))
            }
        );
    }

    #[test]
    fn rebinding_an_action_to_its_own_key_or_a_free_key_is_clean() {
        let m = KeyMap::default();
        assert_eq!(m.conflicts(0, 2, m.slots[0][2]), Conflicts::default());
        assert_eq!(m.conflicts(1, 0, KeyCode::F12), Conflicts::default());
    }
}
