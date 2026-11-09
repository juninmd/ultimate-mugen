#[cfg(target_arch = "wasm32")]
mod web {
    use crate::app::{App, Mode, Screen};
    use js_sys::Reflect;
    use wasm_bindgen::JsValue;

    fn screen_name(screen: &Screen) -> &'static str {
        match screen {
            Screen::Title => "title",
            Screen::Menu { .. } => "menu",
            Screen::Select { .. } => "select",
            Screen::Stages { .. } => "stages",
            Screen::Vs { .. } => "vs",
            Screen::Fight => "fight",
            Screen::Results { .. } => "results",
            Screen::Story { .. } => "story",
            Screen::SurvivalOver { .. } => "survival_over",
            Screen::Controls => "controls",
            Screen::Mugen { .. } => "mugen",
            Screen::Market => "market",
            Screen::Replays { .. } => "replays",
            Screen::Settings { .. } => "settings",
            Screen::Cutscene { .. } => "cutscene",
        }
    }

    fn enabled() -> bool {
        crate::query_params().iter().any(|(k, v)| k == "e2e" && v != "0")
    }

    fn god_requested() -> bool {
        if crate::query_params().iter().any(|(k, v)| k == "e2e" && v == "god") {
            return true;
        }
        Reflect::get(&js_sys::global(), &"__ume_god".into()).map(|v| v.is_truthy()).unwrap_or(false)
    }

    fn fighter_json(f: &ume_engine::fighter::Fighter) -> String {
        format!(
            "{{\"side\":{},\"x\":{:.1},\"lane\":{:.1},\"facing\":{},\"life\":{:.1},\"max\":{:.1},\"state\":{},\"alive\":{}}}",
            f.side, f.pos.x, f.lane, f.facing, f.life, f.life_max, f.state_no, f.alive
        )
    }

    pub fn publish(app: &mut App) {
        thread_local!(static ON: bool = enabled());
        if !ON.with(|on| *on) {
            return;
        }
        let god = god_requested();
        let mut fighters = String::new();
        let mut over = -1i64;
        let mut paused = false;
        if let Some(session) = app.session.as_mut() {
            over = session.over.map(|w| w as i64).unwrap_or(-1);
            paused = session.paused;
            for f in session.m.fighters.iter_mut().filter(|f| !f.is_helper) {
                if god && f.side == 0 && f.alive {
                    f.life = f.life_max;
                }
                if !fighters.is_empty() {
                    fighters.push(',');
                }
                fighters.push_str(&fighter_json(f));
            }
        }
        let story = app
            .story_snapshot()
            .map(|(stage, wave, credits, score, phase)| {
                format!("{{\"stage\":{stage},\"wave\":{wave},\"credits\":{credits},\"score\":{score},\"phase\":\"{phase}\"}}")
            })
            .unwrap_or_else(|| "null".into());
        let json = format!(
            "{{\"screen\":\"{}\",\"story_mode\":{},\"over\":{},\"paused\":{},\"story\":{},\"fighters\":[{}]}}",
            screen_name(&app.screen),
            app.mode == Mode::Story,
            over,
            paused,
            story,
            fighters
        );
        let _ = Reflect::set(&js_sys::global(), &"__ume_state".into(), &JsValue::from_str(&json));
    }
}

pub fn publish(app: &mut crate::app::App) {
    #[cfg(target_arch = "wasm32")]
    web::publish(app);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = app;
}
