use crate::app::{App, Mode, Screen};
use crate::shots::find;
use crate::smoke::ctx::Ctx;

pub async fn run(ctx: &mut Ctx, app: &mut App) {
    let cards = [
        ("menu", Screen::Menu { cur: 0 }),
        ("settings", Screen::Settings { tab: 0, cur: 0, from_fight: false }),
        ("settings-performance", Screen::Settings { tab: 6, cur: 1, from_fight: false }),
        ("controls", Screen::Controls),
        ("mugen-import", Screen::Mugen { view: 0, page: 0 }),
        ("market", Screen::Market),
        ("replays", Screen::Replays { cur: 0 }),
        ("player-card", Screen::PlayerCard { tab: 0 }),
        ("char-tuning", Screen::CharTuning { ch: find(app, "kaito"), cur: 0 }),
    ];
    app.mode = Mode::Cpu;
    for (name, screen) in cards {
        app.screen = screen;
        ctx.shot(app, name, 20).await;
    }
}
