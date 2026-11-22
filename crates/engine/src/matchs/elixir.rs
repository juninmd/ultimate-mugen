use super::power_arena::ARENA_ROOF_Y;
use super::{Event, Match};

const SPOT_X: f32 = 16.0;
const REACH_X: f32 = 26.0;
const REACH_ABOVE: f32 = 70.0;
const REACH_BELOW: f32 = 8.0;
pub const ELIXIR_LIFE_FACTOR: f32 = 2.0;

#[derive(Clone, Debug)]
pub struct Elixir {
    pub x: f32,
    pub y: f32,
    taker: Option<Taker>,
}

#[derive(Clone, Debug)]
struct Taker {
    fighter: usize,
    original_max: f32,
}

impl Elixir {
    pub fn taken(&self) -> bool {
        self.taker.is_some()
    }
}

impl Match {
    pub(super) fn place_elixir(&mut self) {
        self.elixir = Some(Elixir { x: SPOT_X, y: ARENA_ROOF_Y, taker: None });
    }

    pub(super) fn elixir_release(&mut self, fighter: usize) {
        let Some(taker) = self.elixir.as_ref().and_then(|e| e.taker.as_ref()) else {
            return;
        };
        if taker.fighter != fighter {
            return;
        }
        let original = taker.original_max;
        let f = &mut self.fighters[fighter];
        f.life_max = original;
        f.life = f.life.min(original);
        f.ikemen.red_life = f.ikemen.red_life.min(original);
        if let Some(e) = self.elixir.as_mut() {
            e.taker = None;
        }
    }

    pub(super) fn elixir_step(&mut self) {
        let Some(spot) = self.elixir.as_ref().map(|e| (e.x, e.y, e.taken())) else {
            return;
        };
        let (x, y, taken) = spot;
        if self.round_state == 0 {
            if taken {
                let fighter = self.elixir.as_ref().and_then(|e| e.taker.as_ref()).map_or(0, |t| t.fighter);
                self.elixir_release(fighter);
            }
            return;
        }
        if taken || self.round_state != 2 {
            return;
        }
        let toucher = self.fighters.iter().position(|f| {
            !f.is_helper
                && !f.destroyed
                && !f.ikemen.standby
                && f.alive
                && f.life > 0.0
                && (f.pos.x - x).abs() <= REACH_X
                && f.pos.y <= y + REACH_BELOW
                && f.pos.y >= y - REACH_ABOVE
        });
        let Some(i) = toucher else {
            return;
        };
        let f = &mut self.fighters[i];
        let original_max = f.life_max;
        f.life_max *= ELIXIR_LIFE_FACTOR;
        f.life = f.life_max;
        f.ikemen.red_life = f.life_max;
        let id = f.id;
        if let Some(e) = self.elixir.as_mut() {
            e.taker = Some(Taker { fighter: i, original_max });
        }
        self.events.push(Event::Elixir { fighter: id });
        self.sound(i, 5, 0, false, Some("transform"));
    }
}
