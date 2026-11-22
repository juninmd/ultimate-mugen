use crate::tools::{Args, TOOLS};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

const PROTOCOL: &str = "2025-06-18";

fn tool_list() -> Value {
    Value::Array(
        TOOLS
            .iter()
            .map(|t| {
                let props: serde_json::Map<String, Value> = t.params.iter().map(|(k, d, _)| (k.to_string(), json!({"type": "string", "description": d}))).collect();
                let required: Vec<&str> = t.params.iter().filter(|p| p.2).map(|p| p.0).collect();
                json!({"name": t.name, "description": t.description, "inputSchema": {"type": "object", "properties": props, "required": required, "additionalProperties": false}})
            })
            .collect(),
    )
}

fn call(params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
    let Some(tool) = TOOLS.iter().find(|t| t.name == name) else { return json!({"content": [{"type": "text", "text": format!("unknown tool: {name}")}], "isError": true}) };
    let given = params.get("arguments").and_then(Value::as_object).cloned().unwrap_or_default();
    let bad: Vec<&String> = given.keys().filter(|k| !tool.params.iter().any(|p| p.0 == k.as_str())).collect();
    let mut args = Args::new();
    for (k, v) in &given {
        if let Some(s) = v.as_str() {
            args.insert(k.clone(), s.to_string());
        }
    }
    let missing: Vec<&str> = tool.params.iter().filter(|p| p.2 && !args.contains_key(p.0)).map(|p| p.0).collect();
    if !bad.is_empty() || !missing.is_empty() {
        return json!({"content": [{"type": "text", "text": format!("invalid arguments: unknown {bad:?}, missing {missing:?}")}], "isError": true});
    }
    match (tool.run)(&args) {
        Ok(text) => json!({"content": [{"type": "text", "text": text}], "isError": false}),
        Err(err) => json!({"content": [{"type": "text", "text": err}], "isError": true}),
    }
}

pub fn handle(msg: &Value) -> Option<Value> {
    let id = msg.get("id")?.clone();
    let method = msg.get("method").and_then(Value::as_str).unwrap_or_default();
    let result = match method {
        "initialize" => json!({"protocolVersion": msg.pointer("/params/protocolVersion").and_then(Value::as_str).unwrap_or(PROTOCOL), "capabilities": {"tools": {}}, "serverInfo": {"name": "mugen2umf", "version": env!("CARGO_PKG_VERSION")}}),
        "ping" => json!({}),
        "tools/list" => json!({"tools": tool_list()}),
        "tools/call" => call(msg.get("params").unwrap_or(&Value::Null)),
        _ => return Some(json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": format!("method not found: {method}")}})),
    };
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

pub fn serve(input: impl BufRead, mut output: impl Write) {
    for line in input.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(msg) => handle(&msg),
            Err(_) => Some(json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "parse error"}})),
        };
        if let Some(reply) = reply {
            let _ = writeln!(output, "{reply}");
            let _ = output.flush();
        }
    }
}
