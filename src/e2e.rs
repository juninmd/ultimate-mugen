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
            Screen::Tower { .. } => "tower",
            Screen::SurvivalOver { .. } => "survival_over",
            Screen::Controls => "controls",
            Screen::CharTuning { .. } => "char_tuning",
            Screen::Mugen { .. } => "mugen",
            Screen::Market => "market",
            Screen::Replays { .. } => "replays",
            Screen::PlayerCard { .. } => "player_card",
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
            "{{\"side\":{},\"x\":{:.1},\"y\":{:.1},\"ground\":{:.1},\"lane\":{:.1},\"facing\":{},\"life\":{:.1},\"max\":{:.1},\"state\":{},\"alive\":{},\"anim\":{},\"anim_elem\":{},\"form\":{}}}",
            f.side, f.pos.x, f.pos.y, f.ground_level, f.lane, f.facing, f.life, f.life_max, f.state_no, f.alive, f.anim_real, f.elem, f.form
        )
    }

    pub fn publish(app: &mut App) {
        thread_local!(static ON: bool = enabled());
        if !ON.with(|on| *on) {
            return;
        }
        let jump = crate::query_params().iter().find(|(k, _)| k == "e2e_stage").and_then(|(_, v)| v.parse::<usize>().ok());
        if let (Some(stage), Screen::Story { .. }, Some((0, 0, _, 0, _))) = (jump, &app.screen, app.story_snapshot()) {
            app.story_jump_to_stage(stage);
        }
        let cleared = crate::query_params().iter().any(|(k, _)| k == "e2e_clear");
        if let (true, Screen::Story { .. }, Some((0, 0, _, 0, _))) = (cleared, &app.screen, app.story_snapshot()) {
            app.story_clear_preview();
        }
        let ending = crate::query_params().iter().any(|(k, _)| k == "e2e_epilogue");
        if let (true, Screen::Story { .. }, Some((0, 0, _, 0, _))) = (ending, &app.screen, app.story_snapshot()) {
            app.story_epilogue_begin(app.sel.0, 0);
        }
        let god = god_requested();
        let mut fighters = String::new();
        let mut over = -1i64;
        let mut paused = false;
        let mut clock = String::from("null");
        let mut objects = String::from("null");
        if let Some(session) = app.session.as_mut() {
            over = session.over.map(|w| w as i64).unwrap_or(-1);
            paused = session.paused;
            let m = &session.m;
            clock = format!(
                "{{\"tick\":{},\"round\":{},\"round_state\":{},\"round_time\":{},\"zoom\":{:.3},\"winner\":{},\"arena\":{},\"wins\":[{},{}]}}",
                m.game_time, m.round_no, m.round_state, m.round_time, m.zoom, m.winner_side, m.power_arena, m.wins[0], m.wins[1]
            );
            let o = &m.objects;
            objects = format!(
                "{{\"breakables\":{},\"intact\":{},\"pickups\":{},\"goal_ticks\":{}}}",
                o.breakables.len(),
                o.intact_count(),
                o.pickups.len(),
                o.goal_ticks_left().unwrap_or(-1)
            );
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
        let epilogue = app.story_epilogue_page().map_or("null".to_string(), |page| page.to_string());
        let roster = format!(
            "{{\"chars\":{},\"stages\":{},\"errors\":{},\"tail\":[{}]}}",
            app.reg.chars.len(),
            app.reg.stages.len(),
            app.reg.import_errors.len(),
            app.reg.chars.iter().rev().take(3).map(|c| format!("\"{}\"", c.id.replace('"', ""))).collect::<Vec<_>>().join(",")
        );
        let import = format!("{{\"busy\":{},\"queued\":{}}}", crate::webimport::busy(), crate::webimport::queued_imports());
        let wasm_mb = core::arch::wasm32::memory_size(0) * 65536 / (1024 * 1024);
        let st = ume_shell::quad::stats();
        let gpu = format!("{{\"v\":{},\"dc\":{},\"up\":{},\"kb\":{},\"rf\":{:.3},\"if\":{:.3},\"resets\":{},\"tresets\":{}}}", st.vertices, st.draw_calls, st.uploads, st.upload_kb, st.rgba_fill, st.index_fill, st.rgba_resets, st.transient_resets);
        let json = format!(
            "{{\"gpu\":{gpu},\"screen\":\"{}\",\"epilogue\":{epilogue},\"story_mode\":{},\"over\":{},\"paused\":{},\"clock\":{},\"roster\":{roster},\"import\":{import},\"wasm_mb\":{wasm_mb},\"story\":{},\"objects\":{},\"fighters\":[{}]}}",
            screen_name(&app.screen),
            app.mode == Mode::Story,
            over,
            paused,
            clock,
            story,
            objects,
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
