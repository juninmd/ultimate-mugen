use super::{on_off, Act, VOLUME_STEPS};
use crate::gfx::*;
use crate::ui::{hex, Align, Ui};
use std::sync::Arc;

thread_local! {
    static CONTROLS: [Option<Texture2D>; 4] = [
        decode(include_bytes!("../../../assets/ui/settings/toggle-off.png")),
        decode(include_bytes!("../../../assets/ui/settings/toggle-on.png")),
        decode(include_bytes!("../../../assets/ui/settings/step-left.png")),
        decode(include_bytes!("../../../assets/ui/settings/step-right.png")),
    ];
}

fn decode(bytes: &[u8]) -> Option<Texture2D> {
    let image = ume_formats::png::decode_png(bytes).ok()?;
    let rgba = match image.rgba {
        Some(rgba) => rgba,
        None => ume_formats::png::png_indexed_to_rgba(&image),
    };
    Some(Texture2D::from_rgba_shared(
        image.width as u16,
        image.height as u16,
        Arc::new(rgba),
    ))
}

fn draw(ui: &Ui, index: usize, x: f32, y: f32, w: f32, h: f32, alpha: f32) {
    CONTROLS.with(|controls| {
        let Some(texture) = controls[index].as_ref() else {
            return;
        };
        draw_texture_ex(
            texture,
            ui.x(x),
            ui.y(y),
            Color::new(1.0, 1.0, 1.0, alpha),
            DrawTextureParams {
                dest_size: Some(vec2(w * ui.s, h * ui.s)),
                ..Default::default()
            },
        );
    });
}

pub(super) fn toggle(ui: &Ui, x: f32, y: f32, on: bool, alpha: f32) {
    draw(ui, usize::from(on), x, y, 47.0, 30.0, alpha);
}

pub(super) fn step(ui: &Ui, x: f32, y: f32, right: bool, alpha: f32) {
    draw(ui, if right { 3 } else { 2 }, x, y, 23.0, 23.0, alpha);
}

pub(super) fn draw_value(
    ui: &Ui,
    act: Act,
    value: &str,
    level: Option<u8>,
    mute: bool,
    x: f32,
    w: f32,
    y: f32,
    interactive: bool,
    focused: bool,
) {
    if let Some(level) = level {
        let on = if mute { 0 } else { level };
        ui.adapted_level(x + w - 360.0, y + 17.0, VOLUME_STEPS as usize, on as usize);
        ui.text(
            value,
            x + w - 22.0,
            y + 35.0,
            20.0,
            WHITE,
            Align::Right,
            true,
        );
    } else if is_toggle(act) && (value == on_off(true) || value == on_off(false)) {
        toggle(
            ui,
            x + w - 70.0,
            y + 12.0,
            value == on_off(true),
            if interactive { 1.0 } else { 0.5 },
        );
        ui.text(
            value,
            x + w - 86.0,
            y + 35.0,
            20.0,
            WHITE,
            Align::Right,
            true,
        );
    } else if !value.is_empty() {
        let plain = matches!(
            act,
            Act::Keys | Act::CharTuning | Act::StoryStudio | Act::Guide | Act::CheckUpdate | Act::Reset
        );
        let fits_controls =
            measure_text(value, Some(&ui.display), (22.0 * ui.s).max(9.0) as u16, 1.0).width
                <= 140.0 * ui.s;
        let show_controls = interactive && !plain && fits_controls;
        let shown_x = if show_controls {
            x + w - 132.0
        } else {
            x + w - 22.0
        };
        if show_controls {
            let alpha = if focused { 1.0 } else { 0.75 };
            step(ui, x + w - 234.0, y + 16.0, false, alpha);
            step(ui, x + w - 53.0, y + 16.0, true, alpha);
        }
        ui.text(
            value,
            shown_x,
            y + 35.0,
            22.0,
            if focused { WHITE } else { hex("#ffcc33") },
            if show_controls {
                Align::Center
            } else {
                Align::Right
            },
            true,
        );
    }
}

fn is_toggle(act: Act) -> bool {
    matches!(
        act,
        Act::Fullscreen
            | Act::FpsCounter
            | Act::KeepAspect
            | Act::LinearFilter
            | Act::Vsync
            | Act::Mute
            | Act::SingleRound
            | Act::ClassicPushbox
            | Act::TeamLifeShare
            | Act::ShowBoxes
            | Act::StoryEnabled
            | Act::AutoUpdate
    )
}
