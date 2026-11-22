#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pose {
    Idle,
    Reply,
    Arrive,
    Win,
}

pub const ARRIVAL_SECONDS: f32 = 1.4;
const ARRIVAL_FROM: f32 = -120.0;

const IDLE: &[i32] = &[0];
const REPLY: &[i32] = &[195, 181, 0];
const ARRIVE: &[i32] = &[20, 0];
const WIN: &[i32] = &[181, 195, 0];

pub fn anim(pose: Pose, owns: impl Fn(i32) -> bool) -> i32 {
    let chain = match pose {
        Pose::Idle => IDLE,
        Pose::Reply => REPLY,
        Pose::Arrive => ARRIVE,
        Pose::Win => WIN,
    };
    chain.iter().copied().find(|&no| owns(no)).unwrap_or(0)
}

pub fn arriving(t: f32) -> bool {
    t < ARRIVAL_SECONDS
}

pub fn arrival_x(t: f32, home: f32) -> f32 {
    let left = 1.0 - (t / ARRIVAL_SECONDS).clamp(0.0, 1.0);
    ARRIVAL_FROM + (home - ARRIVAL_FROM) * (1.0 - left * left)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poses_use_the_first_animation_the_fighter_owns() {
        assert_eq!(anim(Pose::Reply, |no| no == 195 || no == 181), 195);
        assert_eq!(anim(Pose::Reply, |no| no == 181), 181);
        assert_eq!(anim(Pose::Arrive, |no| no == 20 || no == 0), 20);
        assert_eq!(anim(Pose::Win, |no| no == 195), 195);
    }

    #[test]
    fn poses_fall_back_to_idle_when_nothing_is_owned() {
        for pose in [Pose::Idle, Pose::Reply, Pose::Arrive, Pose::Win] {
            assert_eq!(anim(pose, |_| false), 0);
        }
    }

    #[test]
    fn the_walk_in_ends_exactly_at_the_mark() {
        assert!(arriving(0.0) && !arriving(ARRIVAL_SECONDS));
        assert_eq!(arrival_x(0.0, 380.0), ARRIVAL_FROM);
        assert_eq!(arrival_x(ARRIVAL_SECONDS, 380.0), 380.0);
        assert_eq!(arrival_x(ARRIVAL_SECONDS * 9.0, 380.0), 380.0);
        assert!(arrival_x(0.5, 380.0) < arrival_x(1.0, 380.0));
    }
}
