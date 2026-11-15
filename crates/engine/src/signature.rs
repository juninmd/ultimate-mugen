use crate::fighter::Fighter;
use crate::passives::{AIR_VARIANT_OFFSET, SPECIAL_STATES, SUPER_STATES};

pub const TIER_LIMIT: usize = 4;
pub const SIGNATURE_METER_CAP: f32 = 200.0;
const TICKS_PER_SECOND: f32 = 60.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SignatureTier {
    pub at: f32,
    pub attack: f32,
    pub defence: f32,
    pub speed: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SignatureRule {
    pub max: f32,
    pub gain_hit: f32,
    pub gain_special: f32,
    pub gain_hurt: f32,
    pub decay_delay: i32,
    pub decay_per_tick: f32,
    pub drain_on_super: bool,
    pub risk_per_tick: f32,
    pub tiers: Vec<SignatureTier>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SignatureState {
    pub meter: f32,
    pub idle: i32,
    contact: bool,
    hits: i32,
    state: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SignatureView {
    pub ratio: f32,
    pub tier: usize,
    pub tiers: usize,
}

fn base_state(state_no: i32) -> i32 {
    if state_no >= AIR_VARIANT_OFFSET {
        state_no - AIR_VARIANT_OFFSET
    } else {
        state_no
    }
}

fn is_special(state_no: i32) -> bool {
    let s = base_state(state_no);
    s >= SPECIAL_STATES.0 && s < SPECIAL_STATES.1
}

fn is_super(state_no: i32) -> bool {
    let s = base_state(state_no);
    s >= SUPER_STATES.0 && s < SUPER_STATES.1
}

impl SignatureRule {
    pub fn tier_index(&self, meter: f32) -> usize {
        self.tiers.iter().filter(|t| meter >= t.at).count()
    }

    pub fn tier(&self, meter: f32) -> Option<&SignatureTier> {
        match self.tier_index(meter) {
            0 => None,
            n => self.tiers.get(n - 1),
        }
    }
}

impl Fighter {
    fn signature_scale(&self) -> (f32, f32, f32) {
        let Some(rule) = &self.base_def.passives.signature else {
            return (1.0, 1.0, 1.0);
        };
        if !self.alive {
            return (1.0, 1.0, 1.0);
        }
        match rule.tier(self.passive.signature.meter) {
            Some(t) => (t.attack, t.defence, t.speed),
            None => (1.0, 1.0, 1.0),
        }
    }

    pub fn signature_attack(&self) -> f32 {
        self.signature_scale().0
    }

    pub fn signature_defence(&self) -> f32 {
        self.signature_scale().1
    }

    pub fn signature_speed(&self) -> f32 {
        self.signature_scale().2
    }

    pub fn tick_signature(&mut self, fighting: bool) {
        if self.base_def.passives.signature.is_none() {
            return;
        }
        let def = self.base_def.clone();
        let Some(rule) = def.passives.signature.as_ref() else {
            return;
        };
        let mut st = self.passive.signature;
        let mut gain = 0.0f32;
        let contact = self.move_contact > 0;
        let hurt = (self.received_hits - st.hits).max(0) as f32;
        let state_changed = st.state != self.state_no;
        if contact && !st.contact {
            gain += rule.gain_hit;
        }
        gain += hurt * rule.gain_hurt;
        if state_changed && is_special(self.state_no) {
            gain += rule.gain_special;
        }
        if state_changed && rule.drain_on_super && is_super(st.state) && !is_super(self.state_no) {
            st.meter = 0.0;
        }
        st.contact = contact;
        st.hits = self.received_hits;
        st.state = self.state_no;
        if fighting && self.alive && gain > 0.0 {
            st.meter = (st.meter + gain).min(rule.max);
            st.idle = 0;
        } else if st.meter > 0.0 {
            st.idle += 1;
            if st.idle >= rule.decay_delay {
                st.meter = (st.meter - rule.decay_per_tick).max(0.0);
            }
        }
        if fighting && self.alive && rule.risk_per_tick > 0.0 && st.meter >= rule.max {
            self.life = (self.life - rule.risk_per_tick).max(self.life.min(1.0));
        }
        self.passive.signature = st;
    }

    pub fn signature_view(&self) -> Option<SignatureView> {
        let rule = self.base_def.passives.signature.as_ref()?;
        let meter = self.passive.signature.meter;
        Some(SignatureView {
            ratio: if rule.max > 0.0 { (meter / rule.max).clamp(0.0, 1.0) } else { 0.0 },
            tier: rule.tier_index(meter),
            tiers: rule.tiers.len(),
        })
    }
}

pub fn risk_per_tick(per_second: f32) -> f32 {
    per_second.max(0.0) / TICKS_PER_SECOND
}
