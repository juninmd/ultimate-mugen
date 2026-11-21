use super::types::{NativeChar, NativeMove};

const DEFAULT_CHIP_PERCENT: f32 = 35.0;

fn active_ticks(nc: &NativeChar, m: &NativeMove) -> i32 {
    let Some(anim) = nc.anims.get(&m.anim) else { return 30 };
    let last_active = m.hits.iter().map(|h| h.at).max().unwrap_or(3).max(1) as usize;
    let speed = nc.speed.filter(|s| *s > 1.0).unwrap_or(1.0);
    let ticks: i32 = anim.frames.iter().take(last_active).map(|f| f.ticks.max(1)).sum();
    ((ticks as f32 / speed).round() as i32).max(8) + 4
}

pub(super) fn armor_state(nc: &NativeChar, m: &NativeMove, sid: i32) -> String {
    let Some(armor) = &m.armor else { return String::new() };
    format!(
        "[State {sid}, armor]\ntype = HitOverride\ntrigger1 = Time = 0\nattr = SCA,AA,AP\nslot = 0\nkeepstate = 1\narmorhits = {}\narmorthreshold = {}\narmorchip = {DEFAULT_CHIP_PERCENT}\ntime = {}\n\n",
        armor.hits.max(1), armor.threshold.unwrap_or(0.0), active_ticks(nc, m))
}
