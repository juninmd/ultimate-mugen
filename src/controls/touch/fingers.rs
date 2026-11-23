use super::{BUTTONS, CORE_BUTTONS};
use crate::controls::touch_layout::Layout;
use crate::gfx::{vec2, Vec2};
use ume_engine::input::k;

const SLIDE_RADIUS: f32 = 2.0;
const BUTTON_REACH: f32 = 1.25;
const DEAD_ZONE: f32 = 0.22;
const KEEP_ZONE: f32 = 0.16;
const AXIS_BIAS: f32 = 0.414;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Role {
    Stick,
    Button(usize),
    Toggle,
    Idle,
}

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub id: u64,
    pub pos: Vec2,
    pub ended: bool,
}

#[derive(Clone, Copy, Debug)]
struct Owner {
    id: u64,
    role: Role,
    origin: Vec2,
    pos: Vec2,
    active_dir: bool,
}

#[derive(Default)]
pub struct Fingers {
    owners: Vec<Owner>,
}

#[derive(Default, Debug)]
pub struct Out {
    pub held: [bool; BUTTONS],
    pub released: [bool; BUTTONS],
    pub toggle: bool,
    pub stick: Option<StickOut>,
}

#[derive(Clone, Copy, Debug)]
pub struct StickOut {
    pub origin: Vec2,
    pub offset: Vec2,
    pub bits: u32,
}

fn visible(i: usize, extras: bool) -> bool {
    i < CORE_BUTTONS || extras
}

fn pick_button(p: Vec2, lay: &Layout, extras: bool) -> Option<usize> {
    (0..BUTTONS)
        .filter(|i| visible(*i, extras))
        .map(|i| (i, lay.pos[i].distance(p) / lay.radius[i]))
        .filter(|(_, d)| *d < BUTTON_REACH)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

fn stick_origin(p: Vec2, lay: &Layout, floating: bool) -> Vec2 {
    if !floating {
        return lay.stick;
    }
    let [x0, y0, x1, y1] = lay.zone;
    let pad = lay.stick_radius;
    let clamp = |v: f32, lo: f32, hi: f32| {
        if hi > lo {
            v.clamp(lo, hi)
        } else {
            (lo + hi) / 2.0
        }
    };
    vec2(
        clamp(p.x, x0 + pad, x1 - pad),
        clamp(p.y, y0 + pad, y1 - pad),
    )
}

pub fn stick_bits(d: Vec2, radius: f32, was_active: bool) -> u32 {
    let dead = radius * if was_active { KEEP_ZONE } else { DEAD_ZONE };
    if d.length() <= dead {
        return 0;
    }
    let (ax, ay) = (d.x.abs(), d.y.abs());
    let mut bits = 0;
    if ax > ay * AXIS_BIAS {
        bits |= if d.x > 0.0 { k::R } else { k::L };
    }
    if ay > ax * AXIS_BIAS {
        bits |= if d.y > 0.0 { k::D } else { k::U };
    }
    bits
}

impl Fingers {
    pub fn is_empty(&self) -> bool {
        self.owners.is_empty()
    }

    fn stick_owned(&self) -> bool {
        self.owners.iter().any(|o| o.role == Role::Stick)
    }

    fn classify(&self, p: Vec2, lay: &Layout, extras: bool) -> Role {
        if lay.toggle.distance(p) < lay.toggle_radius * 1.6 {
            return Role::Toggle;
        }
        if let Some(i) = pick_button(p, lay, extras) {
            return Role::Button(i);
        }
        if lay.in_stick_zone(p) && !self.stick_owned() {
            return Role::Stick;
        }
        Role::Idle
    }

    fn finish(role: Role, out: &mut Out) {
        match role {
            Role::Button(i) => out.released[i] = true,
            _ => {}
        }
    }

    pub fn update(
        &mut self,
        samples: &[Sample],
        lay: &Layout,
        extras: bool,
        floating: bool,
    ) -> Out {
        let mut out = Out::default();
        let gone: Vec<Owner> = self
            .owners
            .iter()
            .filter(|o| samples.iter().all(|s| s.id != o.id))
            .copied()
            .collect();
        for o in gone {
            Self::finish(o.role, &mut out);
        }
        self.owners.retain(|o| samples.iter().any(|s| s.id == o.id));
        for o in self.owners.iter_mut() {
            if matches!(o.role, Role::Button(b) if !visible(b, extras)) {
                o.role = Role::Idle;
            }
        }
        for s in samples {
            match self.owners.iter().position(|o| o.id == s.id) {
                None => self.begin(s, lay, extras, floating, &mut out),
                Some(i) if s.ended => {
                    let o = self.owners.swap_remove(i);
                    Self::finish(o.role, &mut out);
                }
                Some(i) => self.follow(i, s.pos, lay, extras, floating),
            }
        }
        for o in &self.owners {
            if let Role::Button(i) = o.role {
                out.held[i] = true;
            }
        }
        out.stick = self.owners.iter().find(|o| o.role == Role::Stick).map(|o| {
            let d = o.pos - o.origin;
            let travel = lay.stick_radius * 0.6;
            let offset = if d.length() > travel {
                d / d.length() * travel
            } else {
                d
            };
            StickOut {
                origin: o.origin,
                offset,
                bits: stick_bits(d, lay.stick_radius, o.active_dir),
            }
        });
        for o in self.owners.iter_mut().filter(|o| o.role == Role::Stick) {
            o.active_dir = out.stick.is_some_and(|s| s.bits != 0);
        }
        out
    }

    fn begin(&mut self, s: &Sample, lay: &Layout, extras: bool, floating: bool, out: &mut Out) {
        let role = self.classify(s.pos, lay, extras);
        out.toggle |= role == Role::Toggle;
        if s.ended {
            Self::finish(role, out);
            return;
        }
        let origin = if role == Role::Stick {
            stick_origin(s.pos, lay, floating)
        } else {
            s.pos
        };
        self.owners.push(Owner {
            id: s.id,
            role,
            origin,
            pos: s.pos,
            active_dir: false,
        });
    }

    fn follow(&mut self, i: usize, p: Vec2, lay: &Layout, extras: bool, floating: bool) {
        let o = &mut self.owners[i];
        o.pos = p;
        match o.role {
            Role::Button(b) if lay.pos[b].distance(p) > lay.radius[b] * SLIDE_RADIUS => {
                o.role = pick_button(p, lay, extras).map_or(Role::Idle, Role::Button);
            }
            Role::Stick if floating => {
                let d = p - o.origin;
                let reach = lay.stick_radius;
                if d.length() > reach {
                    o.origin += d / d.length() * (d.length() - reach);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
