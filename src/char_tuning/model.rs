use ume_engine::chardef::CharDef;

pub const LIFE: usize = 0;
pub const ATTACK: usize = 1;
pub const DEFENCE: usize = 2;
pub const WALK: usize = 3;
pub const RUN: usize = 4;
pub const JUMP: usize = 5;
pub const GRAVITY: usize = 6;
pub const POWER: usize = 7;
pub const POWER_COST: usize = 8;
pub const COUNT: usize = 9;

pub const NEUTRAL: u16 = 100;

pub struct Param {
    pub key: &'static str,
    pub label: (&'static str, &'static str),
    pub min: u16,
    pub max: u16,
    pub step: u16,
}

pub const PARAMS: [Param; COUNT] = [
    Param { key: "life", label: ("Vida", "Life"), min: 25, max: 300, step: 5 },
    Param { key: "attack", label: ("Dano causado", "Damage dealt"), min: 25, max: 300, step: 5 },
    Param { key: "defence", label: ("Dano recebido", "Damage taken"), min: 25, max: 300, step: 5 },
    Param { key: "walk", label: ("Velocidade andando", "Walk speed"), min: 25, max: 300, step: 5 },
    Param { key: "run", label: ("Velocidade correndo", "Run speed"), min: 25, max: 300, step: 5 },
    Param { key: "jump", label: ("Força do pulo", "Jump force"), min: 25, max: 300, step: 5 },
    Param { key: "gravity", label: ("Gravidade", "Gravity"), min: 25, max: 300, step: 5 },
    Param { key: "power", label: ("Barra de poder", "Power bar"), min: 0, max: 300, step: 10 },
    Param { key: "power_cost", label: ("Custo de energia", "Energy cost"), min: 0, max: 300, step: 10 },
];

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Tuning(pub [u16; COUNT]);

impl Default for Tuning {
    fn default() -> Self {
        Tuning([NEUTRAL; COUNT])
    }
}

impl Tuning {
    pub fn is_default(&self) -> bool {
        *self == Tuning::default()
    }

    pub fn apply(&self, d: &mut CharDef) {
        scale(d, &["data.life"], self.0[LIFE]);
        scale(d, &["data.attack"], self.0[ATTACK]);
        d.constants.insert("tuning.defence".into(), self.0[DEFENCE] as f64);
        scale(d, &["velocity.walk.fwd.x", "velocity.walk.back.x"], self.0[WALK]);
        scale(d, &["velocity.run.fwd.x", "velocity.run.back.x"], self.0[RUN]);
        scale(d, &["velocity.jump.y", "velocity.airjump.y"], self.0[JUMP]);
        scale(d, &["movement.yaccel"], self.0[GRAVITY]);
        scale(d, &["data.power"], self.0[POWER]);
        d.constants.insert("tuning.power_cost".into(), self.0[POWER_COST] as f64);
    }
}

fn scale(d: &mut CharDef, keys: &[&str], pct: u16) {
    if pct == NEUTRAL {
        return;
    }
    for key in keys {
        if let Some(v) = d.constants.get_mut(*key) {
            *v *= pct as f64 / NEUTRAL as f64;
        }
    }
}
