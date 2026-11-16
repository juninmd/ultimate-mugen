use crate::app::{App, Screen};
use crate::gfx::*;
use crate::i18n::Lang;
use crate::shots::find;

const FULL_STORIES: [&str; 3] = ["kaito", "bruno", "valdric"];
const TRANSLATED: [(Lang, &str); 2] = [(Lang::En, "en"), (Lang::Es, "es")];
const REVEALED: f32 = 5.0;
const SETTLE_FRAMES: usize = 6;
const BURST_FRAMES: usize = 60;
const BURST_STRIDE: usize = 3;

fn roster() -> Vec<String> {
    let index: Vec<String> =
        serde_json::from_str(include_str!("../../assets/chars/index.json")).unwrap_or_default();
    index
        .into_iter()
        .filter(|id| !id.starts_with("npc_"))
        .collect()
}

async fn frames(app: &mut App, count: usize) {
    for _ in 0..count {
        app.frame().await;
        next_frame().await;
    }
}

async fn shot(app: &mut App, hero: &str, page: usize, path: &str) {
    let index = find(app, hero);
    app.story_epilogue_begin(index, page);
    frames(app, SETTLE_FRAMES).await;
    app.screen = Screen::Story { t: REVEALED };
    app.trans = 0.0;
    app.frame().await;
    get_screen_data().export_png(path);
    println!("shot {path}");
    next_frame().await;
}

fn pages_of(hero: &str) -> usize {
    crate::story::epilogue::of(hero).map_or(0, |script| script.page_count("pt"))
}

fn speak(app: &mut App, lang: Lang) {
    app.settings.lang = lang;
    crate::i18n::set(lang);
}

pub async fn run(app: &mut App, dir: &str) {
    speak(app, Lang::Pt);
    for hero in FULL_STORIES {
        for page in 0..pages_of(hero) {
            shot(
                app,
                hero,
                page,
                &format!("{dir}/epilogue-{hero}-{}.png", page + 1),
            )
            .await;
        }
    }
    for hero in roster() {
        let closing = pages_of(&hero).saturating_sub(2);
        shot(app, &hero, closing, &format!("{dir}/home-{hero}.png")).await;
    }
    for (lang, code) in TRANSLATED {
        speak(app, lang);
        shot(app, "suzume", 1, &format!("{dir}/lang-{code}-suzume.png")).await;
    }
    speak(app, Lang::Pt);
    if std::env::var("UME_TOUR_BURST").is_err() {
        return;
    }
    let index = find(app, "jin");
    app.story_epilogue_begin(index, 2);
    for frame in 0..BURST_FRAMES {
        frames(app, BURST_STRIDE).await;
        get_screen_data().export_png(&format!("{dir}/burst-{frame:02}.png"));
    }
}
