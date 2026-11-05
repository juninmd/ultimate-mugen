pub fn is_hit_var(name: &str) -> bool {
    matches!(
        name,
        "type" | "animtype" | "airtype" | "groundtype" | "damage" | "hitcount" | "guardcount"
        | "hitshaketime" | "hittime" | "slidetime" | "ctrltime" | "recovertime" | "xvel" | "yvel"
        | "yaccel" | "xaccel" | "chainid" | "guarded" | "fall" | "fall.damage" | "fall.xvel"
        | "fall.yvel" | "fall.recover" | "fall.time" | "fall.recovertime" | "fall.kill"
        | "fall.envshake.time" | "fall.envshake.freq" | "fall.envshake.ampl"
        | "fall.envshake.phase" | "kill" | "id" | "playerid" | "playerno" | "down.recover"
        | "priority" | "facing" | "power" | "isbound" | "attr" | "guardflag" | "hitdamage"
        | "guarddamage" | "hitpower" | "guardpower" | "dizzypoints" | "guardpoints" | "redlife"
        | "score" | "teamside" | "projid" | "p1stateno" | "p2stateno"
    )
}

pub fn note_hit_var(name: &str) {
    static REPORTED: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<String>>> = std::sync::OnceLock::new();
    if is_hit_var(name) { return; }
    let first = REPORTED.get_or_init(Default::default).lock().map(|mut set| set.insert(name.to_string())).unwrap_or(false);
    if first { eprintln!("gethitvar: unknown parameter '{name}' evaluates to 0"); }
}