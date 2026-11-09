use crate::json_order::Json;

pub fn round_to(value: f64, decimals: usize) -> f64 {
    format!("{value:.decimals$}").parse().unwrap_or(value)
}

fn number(value: Option<&Json>) -> f64 {
    value.and_then(Json::as_f64).unwrap_or(0.0)
}

pub fn scale_pair(pair: Option<&Json>, factor: f64) -> Json {
    let items = pair.and_then(Json::as_array).cloned().unwrap_or_default();
    Json::Arr(items.iter().map(|x| Json::float(round_to(number(Some(x)) * factor, 2))).collect())
}

pub fn find_move<'a>(data: &'a mut Json, id: &str) -> Result<&'a mut Json, String> {
    data.get_mut("moves")
        .and_then(Json::as_array_mut)
        .and_then(|moves| moves.iter_mut().find(|m| m.get("id").and_then(Json::as_str) == Some(id)))
        .ok_or_else(|| format!("move {id} not found"))
}

fn scaled_int(data: &mut Json, group: &str, key: &str, factor: f64) {
    let value = number(data.get(group).and_then(|s| s.get(key)));
    if let Some(stats) = data.get_mut(group) {
        stats.set(key, Json::int((value * factor).round_ties_even() as i64));
    }
}

fn armor(hits: i64, threshold: i64) -> Json {
    Json::Obj(vec![("hits".into(), Json::int(hits)), ("threshold".into(), Json::int(threshold))])
}

fn velocity_step(x: Json) -> Json {
    Json::Arr(vec![Json::Obj(vec![("at".into(), Json::int(1)), ("vel".into(), Json::Arr(vec![x, Json::int(0)]))])])
}

fn speed_times(data: &mut Json, factor: f64) {
    let speed = data.get("speed").and_then(Json::as_f64).unwrap_or(1.0);
    data.set("speed", Json::float(round_to(speed * factor, 3)));
}

pub fn brute(data: &mut Json) -> Result<(), String> {
    for id in ["strong", "hkick"] {
        find_move(data, id)?.set("armor", armor(1, 90));
    }
    Ok(())
}

pub fn blade(data: &mut Json) -> Result<(), String> {
    speed_times(data, 1.25);
    find_move(data, "hkick")?.set("moves", velocity_step(Json::float(6.5)));
    find_move(data, "lkick")?.set("moves", velocity_step(Json::float(4.5)));
    Ok(())
}

pub fn pyro(data: &mut Json) -> Result<(), String> {
    let mut orb = find_move(data, "starburst")?.get("projectile").cloned().ok_or("starburst has no projectile")?;
    orb.set("damage", Json::int(34));
    orb.set("speed", Json::Arr(vec![Json::float(7.5), Json::int(0)]));
    find_move(data, "jab")?.set("projectile", orb);
    Ok(())
}

pub fn colossus(data: &mut Json) -> Result<(), String> {
    scaled_int(data, "stats", "life", 1.6);
    let (walk, run) = (scale_pair(data.get("stats").and_then(|s| s.get("walk")), 0.78), scale_pair(data.get("stats").and_then(|s| s.get("run")), 0.78));
    if let Some(stats) = data.get_mut("stats") {
        stats.set("walk", walk);
        stats.set("run", run);
    }
    scaled_int(data, "stats", "height", 1.12);
    for id in ["jab", "strong", "lkick", "hkick"] {
        find_move(data, id)?.set("armor", armor(2, 120));
    }
    Ok(())
}

pub fn phantom(data: &mut Json) -> Result<(), String> {
    speed_times(data, 1.15);
    for id in ["lkick", "hkick"] {
        find_move(data, id)?.set("invuln", Json::Arr(vec![Json::int(1), Json::int(3)]));
    }
    find_move(data, "strong")?.set("moves", velocity_step(Json::float(5.0)));
    Ok(())
}
