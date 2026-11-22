use super::screen;
use crate::app::keyedit_note::conflict_note;
use crate::app::{App, KeyEdit, Screen};

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    let key = app.c.keys.slots[1][3];
    let conflicts = app.c.keys.conflicts(0, 0, key);
    let msg = format!("{}: {}{}", crate::keybinds::action_label(0), crate::keybinds::label(key), conflict_note(conflicts));
    app.screen = Screen::Controls;
    app.key_edit = Some(KeyEdit { slot: 0, cur: 0, waiting: false, msg });
    screen(app, dir, "key-conflict-warning", 12).await;
}
