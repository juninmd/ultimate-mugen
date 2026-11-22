use super::*;
use crate::photo::Input;
use ume_engine::photo_camera;

const CAPTURE_KEYS: [KeyCode; 2] = [KeyCode::Enter, KeyCode::Space];
const LEAVE_KEYS: [KeyCode; 3] = [KeyCode::Escape, KeyCode::P, KeyCode::C];

fn down(keys: &[KeyCode]) -> f32 {
    keys.iter().any(|k| is_key_down(*k)) as i32 as f32
}

fn tapped(keys: &[KeyCode]) -> bool {
    keys.iter().any(|k| is_key_pressed(*k))
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub(crate) const CAN_SAVE: bool = true;
#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
pub(crate) const CAN_SAVE: bool = false;

impl App {
    fn photo_input(&self, dt: f32) -> Input {
        let wheel = mouse_wheel().1.clamp(-3.0, 3.0);
        let step = if dt > 0.0 { 0.12 / dt } else { 0.0 };
        let pad = |bit: u32| if self.c.pressed(bit) { step } else { 0.0 };
        Input {
            pan: (
                down(&[KeyCode::Right, KeyCode::D]) - down(&[KeyCode::Left, KeyCode::A]) + pad(nav::RIGHT) - pad(nav::LEFT),
                down(&[KeyCode::Down, KeyCode::S]) - down(&[KeyCode::Up, KeyCode::W]) + pad(nav::DOWN) - pad(nav::UP),
            ),
            zoom: down(&[KeyCode::Equal, KeyCode::KpAdd, KeyCode::PageUp])
                - down(&[KeyCode::Minus, KeyCode::KpSubtract, KeyCode::PageDown])
                + wheel * if dt > 0.0 { 0.25 / dt } else { 0.0 },
            toggle_hud: is_key_pressed(KeyCode::H),
            next_filter: is_key_pressed(KeyCode::F),
            reset: is_key_pressed(KeyCode::R),
        }
    }

    pub(super) fn photo_frame(&mut self, dt: f32) {
        let input = self.photo_input(dt);
        let leave = tapped(&LEAVE_KEYS) || self.c.pressed(nav::BACK);
        let shoot = tapped(&CAPTURE_KEYS) || self.c.pressed(nav::OK);
        let step = is_key_pressed(KeyCode::Period);
        let (Some(photo), Some(s)) = (self.photo.as_mut(), self.session.as_mut()) else {
            self.photo = None;
            return;
        };
        photo.apply(input, dt);
        if step {
            s.step_frame(&mut self.c, &mut self.audio);
        }
        let base = self.g.post.look.clone();
        self.g.post.look = photo.styled_look(&base);
        let level = self.g.post.level;
        self.g.post.level = level.max(2);
        s.hud.hidden = !photo.hud;
        let saved = photo_camera::frame(&mut s.m, &photo.view);
        s.vfx.photo_zoom = photo_camera::closeup(&photo.view);
        s.draw(&mut self.g, &self.ui);
        if let Some(saved) = saved {
            photo_camera::unframe(&mut s.m, saved);
        }
        self.g.post.look = base;
        self.g.post.level = level;
        s.hud.hidden = false;
        s.vfx.photo_zoom = 1.0;
        if shoot {
            self.photo_capture();
        } else {
            self.photo_overlay();
        }
        if leave {
            self.photo = None;
            self.audio.sfx("back");
        }
    }

    fn photo_capture(&mut self) {
        let Some(photo) = self.photo.as_mut() else { return };
        if !CAN_SAVE {
            photo.announce(crate::i18n::tr3("Captura disponível no desktop", "Capture is available on desktop", "Captura disponible en escritorio").to_string());
            return;
        }
        match crate::photo::save_path() {
            Ok(path) => {
                get_screen_data().export_png(&path);
                photo.shutter();
                photo.announce(crate::i18n::tr3("Foto salva: ", "Photo saved: ", "Foto guardada: ").to_string() + &path);
            }
            Err(e) => photo.announce(e),
        }
    }
}
