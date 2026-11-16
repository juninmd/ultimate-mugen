use super::screen;
use crate::app::player_card_edit::CardEdit;
use crate::app::{App, Screen};
use crate::player_card::Record;

fn seeded() -> Record {
    let mut r = Record::default();
    r.card.name = "Ana".into();
    let plan = [
        ("kaito", "garan", true),
        ("kaito", "volta", true),
        ("kaito", "garan", false),
        ("sylene", "marlow", true),
        ("kaito", "sylene", true),
        ("garan", "kaname", true),
        ("sylene", "volta", false),
        ("kaito", "marlow", true),
        ("kaname", "garan", true),
        ("kaito", "volta", true),
        ("volta", "marlow", false),
        ("sylene", "garan", true),
    ];
    for (fighter, rival, won) in plan {
        r.apply_set(won, fighter, rival);
    }
    r
}

pub(super) async fn run(app: &mut App, dir: &str) {
    app.player = seeded();
    let names = ["card", "lutadores", "partidas", "amigos", "conquistas"];
    for (tab, name) in names.iter().enumerate() {
        app.screen = Screen::PlayerCard { tab };
        screen(app, dir, &format!("ficha-{name}"), 20).await;
    }
    app.screen = Screen::PlayerCard { tab: 0 };
    app.card_edit = Some(CardEdit::open(&app.player.card));
    screen(app, dir, "ficha-editar", 20).await;
    app.card_edit = None;
}
