use serde_json::{json, Map, Value};

const META: [&str; 11] = ["name", "description", "color", "series", "author", "intro", "taunt", "winQuote", "profile", "rivals", "element"];

fn route_cns(r: &Map<String, Value>, command: &str) -> Result<String, String> {
    let state = r.get("state").and_then(Value::as_i64).ok_or("mugenRoutes entry needs a numeric `state`")?;
    let name = r.get("name").or(r.get("id")).and_then(Value::as_str).unwrap_or(command);
    let cost = r.get("cost").and_then(Value::as_f64).unwrap_or(0.0);
    let when = match r.get("when").and_then(Value::as_str).unwrap_or("stand") {
        "air" => "statetype = A",
        "crouch" => "statetype = C",
        _ => "statetype != A",
    };
    let cost_line = if cost > 0.0 { format!("triggerall = power >= {cost}\n") } else { String::new() };
    Ok(format!("[State -1, {name}]\ntype = ChangeState\nvalue = {state}\ntriggerall = command = \"{command}\"\n{cost_line}trigger1 = {when} && ctrl\n\n"))
}

fn routes(add: &mut Map<String, Value>) -> Result<(Vec<Value>, String), String> {
    let mut commands = vec![];
    let mut cns = String::new();
    for r in add.remove("mugenRoutes").and_then(|v| v.as_array().cloned()).unwrap_or_default() {
        let r = r.as_object().ok_or("mugenRoutes entries must be objects")?;
        let id = r.get("id").and_then(Value::as_str).ok_or("mugenRoutes entry needs an `id`")?;
        let input = r.get("input").and_then(Value::as_str).ok_or("mugenRoutes entry needs an `input`")?;
        let command = format!("umf_{id}");
        cns += &route_cns(r, &command)?;
        let display = r.get("name").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| id.replace('_', " "));
        commands.push(json!({"name": command, "input": input, "time": 15, "buffer": 1, "display": display, "cost": r.get("cost").cloned().unwrap_or(json!(0)), "state": r.get("state")}));
    }
    Ok((commands, if cns.is_empty() { cns } else { format!("[Statedef -1]\n\n{cns}") }))
}

fn merge_scripts(fighter: &mut Map<String, Value>, theirs: Option<Value>, commands: Vec<Value>, cns: String) {
    let mut scripts = fighter.remove("scripts").and_then(|v| v.as_object().cloned()).unwrap_or_default();
    let mut all_cns: Vec<Value> = match scripts.remove("cns") {
        Some(Value::Array(a)) => a,
        Some(v @ Value::String(_)) => vec![v],
        _ => vec![],
    };
    let mut all_cmds = scripts.remove("commands").and_then(|v| v.as_array().cloned()).unwrap_or_default();
    if let Some(Value::Object(mut t)) = theirs {
        match t.remove("cns") {
            Some(Value::Array(a)) => all_cns.extend(a),
            Some(v @ Value::String(_)) => all_cns.push(v),
            _ => {}
        }
        all_cmds.extend(t.remove("commands").and_then(|v| v.as_array().cloned()).unwrap_or_default());
        scripts.extend(t);
    }
    if !cns.is_empty() {
        all_cns.push(json!(cns));
    }
    all_cmds.extend(commands);
    if !all_cns.is_empty() {
        scripts.insert("cns".into(), Value::Array(all_cns));
    }
    if !all_cmds.is_empty() {
        scripts.insert("commands".into(), Value::Array(all_cmds));
    }
    if !scripts.is_empty() {
        fighter.insert("scripts".into(), Value::Object(scripts));
    }
}

fn split_moves(obj: &mut Map<String, Value>, moves: Vec<Value>) -> Vec<Value> {
    let mut full = vec![];
    for m in moves {
        let (Some(id), Some(name)) = (m.get("id").and_then(Value::as_str), m.get("name").and_then(Value::as_str)) else { continue };
        if m.get("anim").is_some() {
            full.push(m);
            continue;
        }
        let command = id.trim_start_matches("air_").trim_start_matches("crouch_").to_string();
        if let Some(names) = obj.get_mut("mugen").and_then(|v| v.as_object_mut()).map(|m| m.entry("names").or_insert_with(|| json!({}))).and_then(Value::as_object_mut) {
            names.entry(command).or_insert_with(|| json!(name));
        }
    }
    full
}

pub fn apply(fighter: &mut Value, extra: &Value) -> Result<Vec<String>, String> {
    let mut add = extra.as_object().cloned().ok_or("extra must be a JSON object")?;
    let obj = fighter.as_object_mut().ok_or("UMF root must be an object")?;
    if let Some(Value::Array(moves)) = add.remove("moves") {
        let full = split_moves(obj, moves);
        if !full.is_empty() {
            add.insert("moves".into(), Value::Array(full));
        }
    }
    let (commands, cns) = routes(&mut add)?;
    let scripts = add.remove("scripts");
    let had_routes = !commands.is_empty();
    if scripts.is_some() || had_routes {
        merge_scripts(obj, scripts, commands, cns);
    }
    let mut applied: Vec<String> = vec![];
    if had_routes {
        applied.push("mugenRoutes".into());
    }
    for key in META {
        if let Some(v) = add.remove(key) {
            obj.insert(key.into(), v);
            applied.push(key.into());
        }
    }
    for (key, v) in add {
        obj.insert(key.clone(), v);
        applied.push(key);
    }
    Ok(applied)
}
