//! Gamepads with rumble. Native: gilrs (input, generic force-feedback) + direct HID output reports for
//! Sony DualShock 4 / DualSense (rumble + lightbar over USB and Bluetooth). Web: Gamepad API via JS plugin.
use ume_engine::input::k;

/// Physical pad buttons (backend independent) → fighting-game input bits. Pure so the mapping can be unit-tested.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Btn { DUp, DDown, DLeft, DRight, West, North, South, East, R1, R2, L1, L2, Start, R3, Select }
#[allow(dead_code)]
pub fn pad_bits(ax: f32, ay: f32, pressed: impl Fn(Btn) -> bool) -> u32 {
    let mut b = 0u32;
    if ay > 0.5 || pressed(Btn::DUp) { b |= k::U; }
    if ay < -0.5 || pressed(Btn::DDown) { b |= k::D; }
    if ax < -0.5 || pressed(Btn::DLeft) { b |= k::L; }
    if ax > 0.5 || pressed(Btn::DRight) { b |= k::R; }
    // Sony layout: Square=West X(LP), Triangle=North Y(HP), R1=Z; Cross=South A(LK), Circle=East B(HK), R2=C
    if pressed(Btn::West) { b |= k::X; }
    if pressed(Btn::North) { b |= k::Y; }
    if pressed(Btn::R1) { b |= k::Z; }
    if pressed(Btn::South) { b |= k::A; }
    if pressed(Btn::East) { b |= k::B; }
    if pressed(Btn::R2) { b |= k::C; }
    if pressed(Btn::L1) { b |= k::D2; }
    if pressed(Btn::L2) { b |= k::W; }
    if pressed(Btn::Start) { b |= k::S; }
    // awakening: click the right stick (or SELECT)
    if pressed(Btn::R3) || pressed(Btn::Select) { b |= k::T; }
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn right_stick_click_and_select_awaken_and_nothing_else_does() {
        assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::R3), k::T);
        assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::Select), k::T);
        for b in [Btn::West, Btn::North, Btn::South, Btn::East, Btn::R1, Btn::R2, Btn::L1, Btn::L2, Btn::Start, Btn::DUp] {
            assert_eq!(pad_bits(0.0, 0.0, |x| x == b) & k::T, 0, "{b:?} must not awaken");
        }
        assert_eq!(pad_bits(0.0, 0.0, |_| false), 0);
    }
    #[test]
    fn face_buttons_sticks_and_dpad_map_as_documented() {
        assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::West), k::X);
        assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::South), k::A);
        assert_eq!(pad_bits(0.0, 0.0, |b| b == Btn::L1), k::D2);
        assert_eq!(pad_bits(0.9, 0.0, |_| false), k::R);
        assert_eq!(pad_bits(-0.9, 0.9, |_| false), k::L | k::U);
        assert_eq!(pad_bits(0.2, -0.2, |_| false), 0, "dead zone");
        // awakening together with a direction and an attack
        assert_eq!(pad_bits(0.0, -0.9, |b| b == Btn::R3 || b == Btn::East), k::D | k::B | k::T);
    }
}

#[derive(Clone, Debug, Default)]
pub struct PadInfo { pub name: String, pub bits: u32, pub sony: bool, pub connected: bool, pub menu_bits: u32 }

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod native {
    use super::*;
    use gilrs::ff::{BaseEffect, BaseEffectType, EffectBuilder, Replay, Ticks};
    use gilrs::{Axis, Button, EventType, GamepadId, Gilrs};
    use std::time::{Duration, Instant};

    #[derive(Clone, Copy, PartialEq, Debug)]
    enum SonyKind { Ds4, DualSense }
    struct SonyDev { dev: hidapi::HidDevice, kind: SonyKind, bt: bool, seq: u8, color: [u8; 3], until: Option<Instant> }

    pub struct Pads {
        gilrs: Option<Gilrs>,
        order: Vec<GamepadId>,
        hid: Option<hidapi::HidApi>,
        sony: Vec<SonyDev>,
        effects: Vec<(gilrs::ff::Effect, Instant)>,
        last_scan: Instant,
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut crc = !0u32;
        for &b in data { crc ^= b as u32; for _ in 0..8 { crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 }; } }
        !crc
    }

    impl SonyDev {
        fn send(&mut self, strong: u8, weak: u8) {
            let [r, g, b] = self.color;
            let res = match (self.kind, self.bt) {
                (SonyKind::Ds4, false) => {
                    let mut rep = [0u8; 32];
                    rep[0] = 0x05; rep[1] = 0x07; rep[4] = weak; rep[5] = strong; rep[6] = r; rep[7] = g; rep[8] = b;
                    self.dev.write(&rep)
                }
                (SonyKind::Ds4, true) => {
                    let mut rep = [0u8; 79];
                    rep[0] = 0xA2; rep[1] = 0x11; rep[2] = 0xC0; rep[4] = 0x07; rep[7] = weak; rep[8] = strong; rep[9] = r; rep[10] = g; rep[11] = b;
                    let c = crc32(&rep[..75]);
                    rep[75..79].copy_from_slice(&c.to_le_bytes());
                    self.dev.write(&rep[1..])
                }
                (SonyKind::DualSense, false) => {
                    let mut rep = [0u8; 48];
                    rep[0] = 0x02; rep[1] = 0x03; rep[2] = 0x04 | 0x10; rep[3] = weak; rep[4] = strong; rep[39] = 0x02; rep[45] = r; rep[46] = g; rep[47] = b;
                    self.dev.write(&rep)
                }
                (SonyKind::DualSense, true) => {
                    let mut rep = [0u8; 79];
                    rep[0] = 0xA2; rep[1] = 0x31; rep[2] = self.seq << 4; self.seq = (self.seq + 1) & 15; rep[3] = 0x10;
                    rep[4] = 0x03; rep[5] = 0x04 | 0x10; rep[6] = weak; rep[7] = strong; rep[42] = 0x02; rep[48] = r; rep[49] = g; rep[50] = b;
                    let c = crc32(&rep[..75]);
                    rep[75..79].copy_from_slice(&c.to_le_bytes());
                    self.dev.write(&rep[1..])
                }
            };
            let _ = res;
        }
    }

    impl Pads {
        pub fn new() -> Self {
            let gilrs = Gilrs::new().ok();
            let hid = hidapi::HidApi::new().ok();
            let mut p = Pads { gilrs, order: vec![], hid, sony: vec![], effects: vec![], last_scan: Instant::now() - Duration::from_secs(10) };
            p.scan_sony();
            p
        }
        fn scan_sony(&mut self) {
            self.last_scan = Instant::now();
            let Some(api) = self.hid.as_mut() else { return };
            let _ = api.refresh_devices();
            let mut found = vec![];
            for d in api.device_list() {
                if d.vendor_id() != 0x054C { continue; }
                let kind = match d.product_id() { 0x05C4 | 0x09CC | 0x0BA0 => SonyKind::Ds4, 0x0CE6 | 0x0DF2 => SonyKind::DualSense, _ => continue };
                let bt = matches!(d.bus_type(), hidapi::BusType::Bluetooth);
                if self.sony.iter().any(|s| s.dev.get_device_info().map(|i| i.path() == d.path()).unwrap_or(false)) { continue; }
                if let Ok(dev) = d.open_device(api) { found.push(SonyDev { dev, kind, bt, seq: 0, color: [40, 80, 255], until: None }); }
            }
            self.sony.extend(found);
        }
        pub fn update(&mut self) -> Vec<PadInfo> {
            if self.last_scan.elapsed() > Duration::from_secs(3) { self.scan_sony(); }
            let now = Instant::now();
            for s in self.sony.iter_mut() { if s.until.is_some_and(|u| now >= u) { s.until = None; s.send(0, 0); } }
            self.effects.retain(|(_, t)| now < *t);
            let Some(g) = self.gilrs.as_mut() else { return vec![] };
            while let Some(ev) = g.next_event() {
                match ev.event { EventType::Connected => { if !self.order.contains(&ev.id) { self.order.push(ev.id); } } EventType::Disconnected => self.order.retain(|x| *x != ev.id), _ => {} }
            }
            for (id, _) in g.gamepads() { if !self.order.contains(&id) { self.order.push(id); } }
            self.order.iter().map(|&id| {
                let gp = g.gamepad(id);
                let (ax, ay) = (gp.value(Axis::LeftStickX), gp.value(Axis::LeftStickY));
                let b = super::pad_bits(ax, ay, |btn| gp.is_pressed(match btn {
                    Btn::DUp => Button::DPadUp, Btn::DDown => Button::DPadDown, Btn::DLeft => Button::DPadLeft, Btn::DRight => Button::DPadRight,
                    Btn::West => Button::West, Btn::North => Button::North, Btn::South => Button::South, Btn::East => Button::East,
                    Btn::R1 => Button::RightTrigger, Btn::R2 => Button::RightTrigger2, Btn::L1 => Button::LeftTrigger, Btn::L2 => Button::LeftTrigger2,
                    Btn::Start => Button::Start, Btn::R3 => Button::RightThumb, Btn::Select => Button::Select,
                }));
                let name = gp.name().to_string();
                let sony = name.to_ascii_lowercase().contains("sony") || name.contains("DualShock") || name.contains("DualSense") || name.contains("Wireless Controller") || name.contains("PS4") || name.contains("PS5");
                PadInfo { name, bits: b, sony, connected: gp.is_connected(), menu_bits: b }
            }).collect()
        }
        /// strong/weak in 0..1; duration in ms. Pad index follows the connection order.
        pub fn rumble(&mut self, pad: usize, strong: f32, weak: f32, ms: u32) {
            let until = Instant::now() + Duration::from_millis(ms as u64);
            if let Some(s) = self.sony.get_mut(pad) {
                s.send((strong.clamp(0.0, 1.0) * 255.0) as u8, (weak.clamp(0.0, 1.0) * 255.0) as u8);
                s.until = Some(until);
                return;
            }
            let Some(g) = self.gilrs.as_mut() else { return };
            let Some(&id) = self.order.get(pad) else { return };
            if !g.gamepad(id).is_ff_supported() { return; }
            let mut b = EffectBuilder::new();
            let replay = Replay { play_for: Ticks::from_ms(ms), ..Default::default() };
            if strong > 0.0 { b.add_effect(BaseEffect { kind: BaseEffectType::Strong { magnitude: (strong.clamp(0.0, 1.0) * 65535.0) as u16 }, scheduling: replay, ..Default::default() }); }
            if weak > 0.0 { b.add_effect(BaseEffect { kind: BaseEffectType::Weak { magnitude: (weak.clamp(0.0, 1.0) * 65535.0) as u16 }, scheduling: replay, ..Default::default() }); }
            if let Ok(e) = b.gamepads(&[id]).finish(g) { let _ = e.play(); self.effects.push((e, until)); }
        }
        pub fn set_light(&mut self, pad: usize, rgb: [u8; 3]) {
            if let Some(s) = self.sony.get_mut(pad) { s.color = rgb; s.send(0, 0); }
        }
        pub fn sony_count(&self) -> usize { self.sony.len() }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    #[wasm_bindgen::prelude::wasm_bindgen]
    extern "C" {
        fn ume_gp_count() -> i32;
        fn ume_gp_state(i: i32) -> u32;
        fn ume_gp_sony(i: i32) -> i32;
        fn ume_gp_rumble(i: i32, strong: f32, weak: f32, ms: i32);
    }
    pub struct Pads;
    impl Pads {
        pub fn new() -> Self { Pads }
        pub fn update(&mut self) -> Vec<PadInfo> {
            let n = ume_gp_count();
            (0..n).map(|i| { let b = ume_gp_state(i); PadInfo { name: "Gamepad".into(), bits: b, sony: ume_gp_sony(i) != 0, connected: true, menu_bits: b } }).collect()
        }
        pub fn rumble(&mut self, pad: usize, strong: f32, weak: f32, ms: u32) { ume_gp_rumble(pad as i32, strong, weak, ms as i32) }
        pub fn set_light(&mut self, _pad: usize, _rgb: [u8; 3]) {}
        pub fn sony_count(&self) -> usize { 0 }
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use native::Pads;

/// Android / iOS: touch controls only (Bluetooth pads arrive as keyboard events through the OS).
#[cfg(any(target_os = "android", target_os = "ios"))]
mod mobile {
    use super::*;
    pub struct Pads;
    impl Pads {
        pub fn new() -> Self { Pads }
        pub fn update(&mut self) -> Vec<PadInfo> { vec![] }
        pub fn rumble(&mut self, _pad: usize, _s: f32, _w: f32, _ms: u32) {}
        pub fn set_light(&mut self, _pad: usize, _rgb: [u8; 3]) {}
        pub fn sony_count(&self) -> usize { 0 }
    }
}
#[cfg(any(target_os = "android", target_os = "ios"))]
pub use mobile::Pads;
#[cfg(target_arch = "wasm32")]
pub use web::Pads;
