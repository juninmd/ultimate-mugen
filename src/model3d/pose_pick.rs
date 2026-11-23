use super::bindings::Binding;

pub(super) fn pose(
    b: &mut Binding,
    anim_no: i32,
    anim_ticks: i32,
    state_no: i32,
    state_ticks: i32,
) {
    let by_anim = b
        .anim_names
        .get(&anim_no)
        .and_then(|name| b.states.resolve_anim(name));
    match by_anim {
        Some(binding) => b.model.pose_binding(binding, anim_ticks, &mut b.posed),
        None => b
            .model
            .pose_for_state(&b.states, state_no, state_ticks, &mut b.posed),
    }
}
