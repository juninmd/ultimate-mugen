use super::ability_spec::FormSpec;
use super::lookup::find_native;
use super::types::{NativeChar, NativeFrame, NativeMove};
use crate::chardef::{FormInfo, Sprite};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub(super) fn build_forms(nc: &NativeChar, pal_no: i32, with_images: bool) -> Vec<FormInfo> {
    let mut forms = vec![];
    if let Some(spec) = &nc.abilities {
        for f in &spec.forms {
            let target = f.character.as_deref().and_then(find_native).map(|target_nc| {
                Arc::new(super::compile::compile_native_with(&target_nc, pal_no, with_images, None))
            });
            forms.push(FormInfo {
                name: f.name.clone(),
                cost: f.cost.unwrap_or(1000.0).max(0.0),
                attack: f.attack.unwrap_or(1.3),
                defense: f.defense.unwrap_or(1.25),
                target,
            });
        }
    }
    if forms.is_empty() {
        if let Some(name) = &nc.form {
            forms.push(FormInfo {
                name: name.clone(),
                cost: 1000.0,
                attack: 1.3,
                defense: 1.25,
                target: None,
            });
        }
    }
    forms
}

pub fn form_palette(form: &FormSpec, palette: Option<&Value>) -> Option<Value> {
    let pal = form.palette.as_ref()?;
    let mut colors = palette.and_then(|p| p.get("colors").unwrap_or(p).as_object()).cloned().unwrap_or_default();
    if let Some(parts) = &form.parts {
        if let Some(overrides) = pal.get("colors").unwrap_or(pal).as_object() {
            for (k, v) in overrides {
                if parts.iter().any(|p| p == k) {
                    colors.insert(k.clone(), v.clone());
                }
            }
        }
    } else if let Some(overrides) = pal.get("colors").unwrap_or(pal).as_object() {
        colors.extend(overrides.clone());
    }
    colors.insert("_awakening".into(), Value::Bool(true));
    Some(Value::Object(colors))
}

pub(super) fn insert_form_sprites(
    sprites: &mut HashMap<(i32, i32), Sprite>,
    nc: &NativeChar,
    fr: &NativeFrame,
    index: i32,
    with_images: bool,
    palette: Option<&Value>,
) {
    if !with_images || fr.effect.is_some() { return; }
    let Some(spec) = &nc.abilities else { return; };
    for (i, form) in spec.forms.iter().enumerate() {
        if let Some(p) = form_palette(form, palette) {
            let rendered = super::compile::render_frame(nc, fr, true, Some(&p));
            sprites.insert((2 + i as i32, index), Sprite {
                image: rendered.image,
                width: rendered.width,
                height: rendered.height,
                axis_x: rendered.axis_x,
                axis_y: rendered.axis_y,
                scale: rendered.scale,
            });
        }
    }
}

pub(super) fn collect_form_moves<'a>(nc: &'a NativeChar) -> Vec<(&'a NativeMove, Option<String>, u32)> {
    let mut out = vec![];
    let Some(spec) = &nc.abilities else {
        for m in &nc.moves { out.push((m, None, u32::MAX)); }
        return out;
    };
    let has_replace = spec.forms.iter().any(|f| f.replace_moves);
    for m in &nc.moves {
        let cond = if has_replace { Some("form = 0".into()) } else { None };
        let mask = if has_replace { 1 } else { u32::MAX };
        out.push((m, cond, mask));
    }
    for (i, form) in spec.forms.iter().enumerate() {
        let form_no = (i + 1) as u32;
        let cond = if form.replace_moves {
            Some(format!("form = {form_no}"))
        } else {
            Some(format!("form >= {form_no}"))
        };
        for m in &form.moves {
            out.push((m, cond.clone(), 1 << form_no));
        }
    }
    out
}
