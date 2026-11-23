use crate::controls::Controls;
use ume_engine::input::k;
use ume_engine::matchs::Match;

pub fn bits(m: &Match, c: &Controls, auto: bool) -> u32 {
    let intro = m.round_state == 1 && m.round_no == 1 && m.intro_ticks > 0 && !m.intro_skipped;
    if intro && (auto || !c.touch.began.is_empty()) { k::A } else { 0 }
}
