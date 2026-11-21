mod capture;
mod ctx;
mod features;
mod report;
mod stats;

use crate::app::App;
use ctx::Ctx;

pub fn requested() -> bool {
    #[cfg(target_arch = "wasm32")]
    return crate::query_params().iter().any(|(k, v)| k == "smoke" && v != "0");
    #[cfg(not(target_arch = "wasm32"))]
    std::env::args().any(|a| a == "--smoke")
}

pub fn output_dir() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let args: Vec<String> = std::env::args().collect();
        let given = args.iter().position(|a| a == "--smoke").and_then(|i| args.get(i + 1)).filter(|a| !a.starts_with("--"));
        return given.cloned().or_else(|| std::env::var("UME_SMOKE_OUT").ok()).unwrap_or_else(|| "smoke-report".into());
    }
    #[cfg(target_arch = "wasm32")]
    String::new()
}

pub async fn run_and_exit(app: &mut App) {
    let passed = run(app, &output_dir()).await;
    #[cfg(not(target_arch = "wasm32"))]
    std::process::exit(if passed { 0 } else { 1 });
    #[cfg(target_arch = "wasm32")]
    let _ = passed;
}

async fn run(app: &mut App, out: &str) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    install_panic_report(out);
    let mut ctx = Ctx::new(out);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = std::fs::create_dir_all(format!("{out}/shots"));
    let only = selected_features();
    for feature in features::ALL {
        if only.as_ref().is_some_and(|ids| !ids.iter().any(|id| id == feature.id)) {
            continue;
        }
        ctx.begin(feature.id, feature.title);
        (feature.run)(&mut ctx, app).await;
        ctx.end();
    }
    ctx.flush(true, None);
    finish(&ctx)
}

fn selected_features() -> Option<Vec<String>> {
    #[cfg(not(target_arch = "wasm32"))]
    return std::env::var("UME_SMOKE_ONLY").ok().map(|v| v.split(',').map(str::to_string).collect());
    #[cfg(target_arch = "wasm32")]
    crate::query_params().iter().find(|(k, _)| k == "smoke_only").map(|(_, v)| v.split(',').map(str::to_string).collect())
}

fn finish(ctx: &Ctx) -> bool {
    let json: serde_json::Value = serde_json::from_str(&report::latest()).unwrap_or_default();
    #[cfg(not(target_arch = "wasm32"))]
    write_report(&ctx.out, &json);
    #[cfg(target_arch = "wasm32")]
    let _ = ctx;
    json["passed"].as_bool().unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
fn write_report(out: &str, json: &serde_json::Value) {
    let _ = std::fs::write(format!("{out}/report.json"), serde_json::to_string_pretty(json).unwrap_or_default());
    let _ = std::fs::write(format!("{out}/report.md"), report::markdown(json));
    println!("{}", report::markdown(json));
}

#[cfg(not(target_arch = "wasm32"))]
fn install_panic_report(out: &str) {
    let out = out.to_string();
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut json: serde_json::Value = serde_json::from_str(&report::latest()).unwrap_or_default();
        json["panic"] = serde_json::Value::String(info.to_string());
        json["passed"] = serde_json::Value::Bool(false);
        write_report(&out, &json);
        previous(info);
    }));
}
