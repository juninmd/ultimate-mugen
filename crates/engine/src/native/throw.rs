use super::types::NativeMove;

const VICTIM_STATE_BASE: i32 = 3800;

pub(super) fn victim_state(sid: i32) -> i32 { VICTIM_STATE_BASE + (sid % 100) + if sid >= 3000 { 50 } else { 0 } }

pub(super) fn throw_states(m: &NativeMove, sid: i32) -> (String, String) {
    let Some(throw) = &m.throw else { return (String::new(), String::new()) };
    let grab = throw.at.unwrap_or(1).max(1);
    let victim = throw.p2_state.unwrap_or_else(|| victim_state(sid));
    let bind = throw.bind.unwrap_or([34.0, 0.0]);
    let launch = throw.launch.unwrap_or([-5.0, -6.0]);
    let release = grab + 2;
    let out = format!(
        "[State {sid}, throw_grab]\ntype = HitDef\ntrigger1 = AnimElem = {grab}\nattr = S, NT\nhitflag = M-\nguardflag =\npriority = 7, Miss\npausetime = 0, 0\nsparkno = -1\nsparkxy = 0, -60\nhitsound = 5, 3\np2stateno = {victim}\np2facing = 1\nground.hittime = 60\n\n\
         [State {sid}, throw_bind]\ntype = TargetBind\ntrigger1 = AnimElem = {} || AnimElem = {release}\ntime = 4\npos = {}, {}\n\n\
         [State {sid}, throw_damage]\ntype = TargetLifeAdd\ntrigger1 = AnimElem = {release}\nvalue = -{}\nkill = 1\n\n\
         [State {sid}, throw_launch]\ntype = TargetState\ntrigger1 = AnimElem = {release}\nvalue = 5050\n\n\
         [State {sid}, throw_velocity]\ntype = TargetVelSet\ntrigger1 = AnimElem = {release}\nx = {}\ny = {}\n\n\
         [State {sid}, throw_shake]\ntype = EnvShake\ntrigger1 = AnimElem = {release}\ntime = 12\nfreq = 90\nampl = -5\n\n",
        grab + 1, bind[0], bind[1], throw.damage as i32, launch[0], launch[1]);
    let mut victim_def = String::new();
    if throw.p2_state.is_none() {
        victim_def = format!("[Statedef {victim}]\ntype = A\nmovetype = H\nphysics = N\nanim = 5030\nctrl = 0\nvelset = 0, 0\nsprpriority = 0\n\n[State {victim}, safety]\ntype = ChangeState\ntrigger1 = Time >= 45\nvalue = 5050\nctrl = 0\n\n");
    }
    (out, victim_def)
}
