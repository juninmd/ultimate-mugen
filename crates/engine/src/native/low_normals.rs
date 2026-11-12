use super::types::{NativeAnim, NativeChar, NativeFrame, NativeMove, VelChange};
use serde_json::Value;

const DOWN_LIGHT_KICK: &str = "D+a";
const AIR_FORWARD_SPEED: f32 = 2.2;
const AIR_DOWN_SPEED: f32 = 2.2;

fn move_by_id(nc: &NativeChar, id: &str) -> Option<NativeMove> {
    nc.moves.iter().find(|m| m.id == id).cloned()
}

fn animation(nc: &NativeChar, id: &str) -> Option<NativeAnim> {
    nc.anims.get(id).cloned()
}

fn air_low_animation(nc: &NativeChar) -> Option<NativeAnim> {
    let kick = animation(nc, "akick")?;
    let mut start = kick.frames.first()?.clone();
    start.ticks = 3;
    start.hit = None;
    let mut prep = kick.frames.get(1)?.clone();
    prep.ticks = 2;
    prep.hit = None;
    let mut strike = prep.clone();
    strike.ticks = 4;
    strike.sprite = Some("airLowKick".into());
    strike.hit = kick.frames.get(1)?.hit.clone();
    let landing = animation(nc, "jumpDown")?;
    let mut recovery = landing.frames.first()?.clone();
    recovery.ticks = 5;
    recovery.hit = None;
    let mut fall = landing.frames.last()?.clone();
    fall.ticks = -1;
    fall.hit = None;
    Some(NativeAnim {
        frames: vec![start, prep, strike, recovery, fall],
        loop_start: None,
    })
}

fn add_animation(nc: &mut NativeChar, id: &str, source: &str) {
    if nc.anims.get(id).is_none() {
        if id == "lowKick" {
            if let Some(anim) = low_kick_animation(nc, source) {
                nc.anims.0.push((id.into(), anim));
            }
        } else if let Some(anim) = animation(nc, source) {
            nc.anims.0.push((id.into(), anim));
        }
    }
}

fn set_sprite(frame: &mut NativeFrame, sprite: &str) {
    frame.sprite = Some(sprite.into());
    frame.pose = None;
}

fn low_kick_animation(nc: &NativeChar, source: &str) -> Option<NativeAnim> {
    let base = animation(nc, source)?;
    let mut windup = base.frames.first()?.clone();
    set_sprite(&mut windup, "crouch0");
    windup.hit = None;
    let mut strike = base.frames.get(1)?.clone();
    set_sprite(&mut strike, "lowKick");
    let mut recovery = base.frames.get(2)?.clone();
    set_sprite(&mut recovery, "crouch1");
    recovery.hit = None;
    let mut settle = base.frames.last()?.clone();
    set_sprite(&mut settle, "crouch");
    settle.hit = None;
    Some(NativeAnim {
        frames: vec![windup, strike, recovery, settle],
        loop_start: None,
    })
}

fn replace_inline_recovery(nc: &mut NativeChar, animation: &str, frame: usize, sprite: &str) {
    let Some(frame) = nc
        .anims
        .0
        .iter_mut()
        .find(|(name, _)| name == animation)
        .and_then(|(_, animation)| animation.frames.get_mut(frame))
    else {
        return;
    };
    if frame.sprite.is_none() && frame.pose.as_ref().is_some_and(Value::is_object) {
        set_sprite(frame, sprite);
        frame.hit = None;
    }
}

fn low_move(base: &NativeMove, id: &str, name: &str, anim: &str, when: &str) -> NativeMove {
    let mut m = base.clone();
    m.id = id.into();
    m.name = name.into();
    m.input = DOWN_LIGHT_KICK.into();
    m.simple = None;
    m.anim = anim.into();
    m.when = Some(when.into());
    m.cancels = None;
    for hit in &mut m.hits {
        hit.guard = Some("low".into());
        hit.level = Some("light".into());
        if id == "airlowkick" {
            hit.at = 3;
        }
    }
    m
}

fn air_low_move(base: &NativeMove) -> NativeMove {
    let mut m = low_move(base, "airlowkick", "Air Low Kick", "airLowKick", "air");
    m.moves = vec![VelChange {
        at: 1,
        vel: [Some(AIR_FORWARD_SPEED), Some(AIR_DOWN_SPEED)],
    }];
    m
}

fn taken(nc: &NativeChar, id: &str, when: &str) -> bool {
    nc.moves
        .iter()
        .any(|m| m.id == id || (m.input == DOWN_LIGHT_KICK && m.when.as_deref() == Some(when)))
}

pub(super) fn complete(mut nc: NativeChar) -> NativeChar {
    replace_inline_recovery(&mut nc, "clk", 2, "crouch1");
    replace_inline_recovery(&mut nc, "ajab", 2, "jumpDown");
    replace_inline_recovery(&mut nc, "akick", 2, "jumpDown");
    if let Some(base) = move_by_id(&nc, "clk").filter(|m| m.when.as_deref() == Some("crouch")) {
        if !taken(&nc, "lowkick", "crouch") {
            add_animation(&mut nc, "lowKick", &base.anim);
            nc.moves
                .push(low_move(&base, "lowkick", "Low Kick", "lowKick", "crouch"));
        }
    }
    if let Some(base) = move_by_id(&nc, "akick").filter(|m| m.when.as_deref() == Some("air")) {
        if !taken(&nc, "airlowkick", "air") {
            if nc.anims.get("airLowKick").is_none() {
                if let Some(anim) = air_low_animation(&nc) {
                    nc.anims.0.push(("airLowKick".into(), anim));
                }
            }
            nc.moves.push(air_low_move(&base));
        }
    }
    nc
}

#[cfg(test)]
#[path = "low_normals/tests.rs"]
mod tests;
