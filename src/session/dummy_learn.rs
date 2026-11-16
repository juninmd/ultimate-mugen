//! Training dummy that learns the player: counts how the player's action changes and answers the likeliest next one.
use ume_engine::input::k;

const ATTACK_BITS: u32 = k::A | k::B | k::C | k::X | k::Y | k::Z;
const DECAY_AT: u32 = 64;
const PULSE: u32 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Idle,
    Forward,
    Back,
    Crouch,
    Jump,
    Attack,
}

impl Action {
    pub const ALL: [Action; 6] = [
        Action::Idle,
        Action::Forward,
        Action::Back,
        Action::Crouch,
        Action::Jump,
        Action::Attack,
    ];
}

pub fn classify(bits: u32, facing: f32) -> Action {
    let (forward, back) = if facing >= 0.0 {
        (k::R, k::L)
    } else {
        (k::L, k::R)
    };
    if bits & ATTACK_BITS != 0 {
        Action::Attack
    } else if bits & k::U != 0 {
        Action::Jump
    } else if bits & k::D != 0 {
        Action::Crouch
    } else if bits & back != 0 {
        Action::Back
    } else if bits & forward != 0 {
        Action::Forward
    } else {
        Action::Idle
    }
}

#[derive(Clone, Default)]
pub struct PlayerModel {
    transitions: [[u32; 6]; 6],
    last: Option<Action>,
    frame: u32,
}

impl PlayerModel {
    pub fn observe(&mut self, bits: u32, facing: f32) {
        let now = classify(bits, facing);
        self.frame = self.frame.wrapping_add(1);
        if let Some(prev) = self.last.filter(|&p| p != now) {
            let row = &mut self.transitions[prev as usize];
            row[now as usize] += 1;
            if row.iter().sum::<u32>() >= DECAY_AT {
                row.iter_mut().for_each(|c| *c /= 2);
            }
        }
        self.last = Some(now);
    }

    pub fn predict(&self) -> Option<Action> {
        let row = &self.transitions[self.last? as usize];
        if row.iter().all(|&n| n == 0) {
            return None;
        }
        let best = (0..row.len()).max_by_key(|&i| (row[i], std::cmp::Reverse(i)))?;
        Some(Action::ALL[best])
    }

    pub fn counter_bits(&self, facing: f32) -> u32 {
        let away = if facing >= 0.0 { k::L } else { k::R };
        let pulse = if self.frame % PULSE < PULSE / 2 {
            k::A
        } else {
            0
        };
        match self.predict() {
            Some(Action::Attack) => away,
            Some(Action::Forward | Action::Crouch | Action::Jump) => pulse,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RIGHT: f32 = 1.0;

    fn learn_cycles(model: &mut PlayerModel, habit: u32, cycles: usize) {
        for _ in 0..cycles {
            model.observe(0, RIGHT);
            model.observe(habit, RIGHT);
        }
        model.observe(0, RIGHT);
    }

    #[test]
    fn back_means_away_from_the_rival_whichever_side_the_player_faces() {
        assert_eq!(classify(k::L, RIGHT), Action::Back);
        assert_eq!(classify(k::L, -1.0), Action::Forward);
    }

    #[test]
    fn predicts_the_habit_the_player_repeats_from_idle() {
        let mut model = PlayerModel::default();
        assert_eq!(model.predict(), None);
        learn_cycles(&mut model, k::A, 10);
        assert_eq!(model.predict(), Some(Action::Attack));
    }

    #[test]
    fn follows_a_changed_habit_instead_of_the_old_one() {
        let mut model = PlayerModel::default();
        learn_cycles(&mut model, k::A, 20);
        learn_cycles(&mut model, k::U, 60);
        assert_eq!(model.predict(), Some(Action::Jump));
    }

    #[test]
    fn guards_away_when_an_attack_is_expected_and_punishes_a_jump() {
        let mut model = PlayerModel::default();
        learn_cycles(&mut model, k::A, 10);
        assert_eq!(model.counter_bits(RIGHT), k::L);
        assert_eq!(model.counter_bits(-1.0), k::R);

        let mut jumper = PlayerModel::default();
        learn_cycles(&mut jumper, k::U, 10);
        let pulses: Vec<u32> = (0..PULSE)
            .map(|_| {
                let bits = jumper.counter_bits(RIGHT);
                jumper.observe(0, RIGHT);
                bits
            })
            .collect();
        assert!(pulses.contains(&k::A) && pulses.contains(&0), "{pulses:?}");
    }
}
