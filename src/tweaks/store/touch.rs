use toml_edit::{value, DocumentMut, Item, Table};

pub fn update_touch(scale: f32, opacity: f32, left_handed: bool, floating_stick: bool) {
    let Some(text) = crate::storage::read("tweaks.toml") else {
        return;
    };
    let Ok(mut document) = text.parse::<DocumentMut>() else {
        return;
    };
    let touch = document.entry("touch").or_insert(Item::Table(Table::new()));
    let Some(table) = touch.as_table_like_mut() else {
        return;
    };
    table.insert("scale", value(scale as f64));
    table.insert("opacity", value(opacity as f64));
    table.insert("left_handed", value(left_handed));
    table.insert("floating_stick", value(floating_stick));
    crate::storage::write("tweaks.toml", &document.to_string());
    super::reload_now();
}
