use super::arena_setup::pool_cap;
use super::team::Team;
use super::*;
use crate::i18n::tr3;

impl App {
    pub(crate) fn arena_take_pick(&mut self, cur: [usize; 2]) {
        let me = cur[0];
        let stay = Screen::Select { cur, picking: 0, picked: [None, None] };
        if self.team_pick.len() >= self.team_n && !self.arena.random {
            self.arena.foe = me;
            self.arena_launch();
            return;
        }
        if self.team_pick.contains(&me) {
            self.msg = tr3("Este lutador já está na equipe.", "This fighter is already on the team.", "Este luchador ya está en el equipo.").into();
            self.screen = stay;
            return;
        }
        self.team_pick.push(me);
        self.note_recent(me);
        self.team_pal.push(self.pal_pick.clamp(1, crate::settings::PALETTE_COUNT as i32));
        if self.team_pick.len() < self.team_n {
            self.screen = stay;
        } else if self.arena.random {
            self.arena_launch();
        } else {
            self.msg = tr3("Agora escolha o inimigo.", "Now choose the enemy.", "Ahora elige al enemigo.").into();
            self.screen = stay;
        }
    }

    fn arena_launch(&mut self) {
        self.arena_build_team();
        self.screen = Screen::Vs { t: 0.0, step: 0 };
    }

    fn arena_build_team(&mut self) {
        let foes = self.arena_foe_slots();
        let team = Team::new(self.team_pick.clone(), foes, true);
        self.sel = (team.members[0][0], team.members[1][0], self.sel.2);
        self.team = Some(team);
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) fn arena_link_params(&mut self, enemies: Option<usize>, squad: Option<usize>) {
        if let Some(n) = enemies {
            self.arena.total = n.clamp(1, ume_engine::matchs::SWARM_MAX_TOTAL as usize);
        }
        if let Some(n) = squad {
            self.arena.squad = n.clamp(1, super::arena_setup::SQUADS.len());
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) fn arena_link_team(&mut self) {
        if self.mode != Mode::PowerArena || (self.arena.total <= 1 && self.arena.squad <= 1) {
            return;
        }
        self.arena.random = true;
        let lead = self.sel.0;
        let mates = self.playable().into_iter().filter(|&i| i != lead && !self.reg.chars[i].broken.get() && !self.reg.chars[i].boss);
        self.team_pick = std::iter::once(lead).chain(mates.take(self.arena.squad - 1)).collect();
        self.team_n = self.team_pick.len();
        self.team_pal = vec![1; self.team_pick.len()];
        self.arena_build_team();
    }

    fn arena_foe_slots(&self) -> Vec<usize> {
        let slots = self.arena.slots();
        if !self.arena.random {
            return vec![self.arena.foe; slots];
        }
        let mut pool: Vec<usize> = self
            .playable()
            .into_iter()
            .filter(|i| !self.team_pick.contains(i) && !self.reg.chars[*i].broken.get() && !self.reg.chars[*i].boss)
            .collect();
        if pool.is_empty() {
            pool = self.team_pick.clone();
        }
        let mut seed = (get_time() * 1000.0) as u32 | 1;
        let mut roll = move || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as usize
        };
        let mut distinct = Vec::new();
        while distinct.len() < pool_cap().min(pool.len()) {
            let pick = pool.swap_remove(roll() % pool.len());
            distinct.push(pick);
        }
        (0..slots).map(|k| distinct[k % distinct.len()]).collect()
    }
}
