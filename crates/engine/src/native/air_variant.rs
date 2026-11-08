use super::types::NativeMove;

pub(crate) const AIR_STATE_OFFSET: i32 = 20000;
const DRIFT_KEPT: f32 = 0.4;
const FALL_STARTS_AT_TICK: i32 = 12;
const FALL_GRAVITY_SHARE: f32 = 0.45;
const MINIMUM_TICKS_BEFORE_LANDING: i32 = 2;
const JUMP_FALL_STATE: i32 = 50;
const LANDING_STATE: i32 = 52;
const PROJECTILE_DIVE_SLOPE: f32 = 0.85;
const USED_VAR: i32 = 57;
const AIR_DAMAGE_SHARE: f32 = 0.7;
const DROPPED_BLOCKS: [&str; 4] = [", grav]", ", land]", ", fix]", ", end]"];

pub(super) fn state_id(ground_state: i32) -> i32 {
    ground_state + AIR_STATE_OFFSET
}

pub(super) fn reset_rule() -> String {
    format!("[State -2, air_special_ready]\ntype = VarSet\ntrigger1 = var({USED_VAR}) != 0 && statetype != A\nv = {USED_VAR}\nvalue = 0\n\n")
}

pub(super) fn eligible(m: &NativeMove, when: &str) -> bool {
    matches!(m.kind.as_str(), "special" | "super") && when == "stand" && m.throw.is_none() && m.flight != Some(true)
}

pub(super) fn entry(chunk: &str, ground_state: i32, ground_conditions: &str) -> String {
    chunk
        .replace(&format!("value = {ground_state}\n"), &format!("value = {}\n", state_id(ground_state)))
        .replace(&format!("triggerall = {ground_conditions}"), &format!("triggerall = statetype = A\ntriggerall = var({USED_VAR}) = 0"))
        .replacen("[State -1, ", "[State -1, air ", 1)
}

fn weaken(line: &str) -> String {
    let Some(values) = line.strip_prefix("damage = ") else { return line.to_string() };
    let scaled: Vec<String> = values.split(',').map(|value| value.trim().parse::<f32>().map_or(value.trim().to_string(), |v| format!("{}", (v * AIR_DAMAGE_SHARE).round()))).collect();
    format!("damage = {}", scaled.join(", "))
}

fn dive_projectile(block: &str) -> String {
    let block = block.lines().map(weaken).collect::<Vec<_>>().join("\n");
    if !block.contains("type = Projectile") { return block; }
    block.lines().map(|line| match line.strip_prefix("velocity = ").and_then(|pair| pair.split_once(", ")) {
        Some((x, y)) => match (x.trim().parse::<f32>(), y.trim().parse::<f32>()) {
            (Ok(x), Ok(y)) => format!("velocity = {x}, {}", y + x.abs() * PROJECTILE_DIVE_SLOPE),
            _ => line.to_string(),
        },
        None => line.to_string(),
    }).collect::<Vec<_>>().join("
")
}

pub(super) fn body(chunk: &str, ground_state: i32) -> String {
    let air = state_id(ground_state);
    let mut blocks = chunk.split("\n\n").filter(|block| !block.trim().is_empty());
    let Some(header) = blocks.next() else { return String::new() };
    let header: Vec<String> = header
        .lines()
        .filter(|line| !line.starts_with("velset"))
        .map(|line| match line.split_once(" = ") {
            Some(("type", "S" | "C")) => "type = A".to_string(),
            Some(("physics", _)) => "physics = N".to_string(),
            _ => line.replace(&format!("[Statedef {ground_state}]"), &format!("[Statedef {air}]")),
        })
        .collect();
    let rest: Vec<String> = blocks
        .filter(|block| !DROPPED_BLOCKS.iter().any(|suffix| block.lines().next().is_some_and(|first| first.ends_with(suffix))))
        .map(|block| dive_projectile(&block.replacen(&format!("[State {ground_state},"), &format!("[State {air},"), 1)))
        .collect();
    let drift = format!("[State {air}, drift]\ntype = VelSet\ntrigger1 = Time = 0\nx = Vel X * {DRIFT_KEPT}\ny = 0");
    let fall = format!("[State {air}, fall]\ntype = VelAdd\ntrigger1 = Time >= {FALL_STARTS_AT_TICK}\ny = const(movement.yaccel) * {FALL_GRAVITY_SHARE}");
    let land = format!("[State {air}, land]\ntype = ChangeState\ntrigger1 = Time > {MINIMUM_TICKS_BEFORE_LANDING} && Pos Y >= 0 && Vel Y > 0\nvalue = {LANDING_STATE}");
    let end = format!("[State {air}, end]\ntype = ChangeState\ntrigger1 = AnimTime = 0\nvalue = {JUMP_FALL_STATE}\nctrl = 1");
    let used = format!("[State {air}, used]\ntype = VarSet\ntrigger1 = Time = 0\nv = {USED_VAR}\nvalue = 1");
    let mut parts = vec![header.join("\n"), used, drift];
    parts.extend(rest);
    parts.extend([fall, land, end]);
    parts.join("\n\n") + "\n\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    const GROUND: &str = "[Statedef 1001]\ntype = S\nmovetype = A\nphysics = N\nanim = 1012\nctrl = 0\nvelset = 0, 0\npoweradd = 30\n\n[State 1001, grav]\ntype = VelAdd\ntrigger1 = 1\ny = 1\n\n[State 1001, hit]\ntype = HitDef\ntrigger1 = AnimElem = 3\n\n[State 1001, end]\ntype = ChangeState\ntrigger1 = AnimTime = 0\nvalue = 0\nctrl = 1\n\n";

    #[test]
    fn the_air_body_hovers_lands_and_returns_to_the_jump() {
        let air = body(GROUND, 1001);
        assert!(air.starts_with("[Statedef 21001]\ntype = A\nmovetype = A\nphysics = N"), "{air}");
        assert!(!air.contains("velset") && !air.contains("[State 1001,"), "{air}");
        assert!(air.contains("[State 21001, hit]") && air.contains("[State 21001, land]\ntype = ChangeState"), "{air}");
        assert!(air.contains("value = 50\nctrl = 1") && !air.contains("value = 0\nctrl = 1"), "{air}");
    }

    #[test]
    fn air_projectiles_dive_toward_the_ground() {
        let ground = "[Statedef 1000]
type = S
physics = S

[State 1000, proj]
type = Projectile
projid = 1000
velocity = 6, 0
projhits = 1

";
        assert!(body(ground, 1000).contains("velocity = 6, 5.1"));
    }

    #[test]
    fn the_entry_only_fires_in_the_air() {
        let ground = "[State -1, Fire]\ntype = ChangeState\nvalue = 1001\ntriggerall = command = \"m_fire\"\ntriggerall = statetype = S\ntrigger1 = ctrl\n\n";
        let air = entry(ground, 1001, "statetype = S");
        assert!(air.contains("value = 21001\n") && air.contains("triggerall = statetype = A\ntriggerall = var(57) = 0") && air.starts_with("[State -1, air Fire]"), "{air}");
    }
}
