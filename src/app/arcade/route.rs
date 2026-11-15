use crate::app::{App, Mode};
use crate::route::{load_route, plan_route, unlock_rules, Action, Context, FightSpec};

impl App {
    fn route_context(&self) -> Context {
        let mode = match self.mode {
            Mode::Arcade => "arcade",
            Mode::Story => "story",
            Mode::Survival => "survival",
            Mode::Cpu => "cpu",
            Mode::Versus => "versus",
            Mode::Training => "training",
            Mode::Watch => "watch",
            _ => "team",
        };
        Context::default()
            .with_int("stats.arcade.clear", self.boss_progression.clears as i64)
            .with_text("mode", mode)
            .with_int("netplay", 0)
            .with_text("player", &self.player_char_id())
    }

    pub(crate) fn route_allows(&self, char_index: usize) -> bool {
        let id = self.reg.chars[char_index]
            .id
            .strip_prefix("umf:")
            .unwrap_or(&self.reg.chars[char_index].id);
        unlock_rules().allows(&format!("char:{id}"), &self.route_context())
    }

    fn route_opponent(
        &self,
        fight: &FightSpec,
        pool: &[usize],
        boss: Option<usize>,
        taken: usize,
    ) -> Option<usize> {
        if let Some(names) = &fight.p2 {
            let found = names.iter().find_map(|wanted| {
                self.reg.chars.iter().position(|c| {
                    let id = c.id.strip_prefix("umf:").unwrap_or(&c.id);
                    id.eq_ignore_ascii_case(wanted) || c.name.eq_ignore_ascii_case(wanted)
                })
            });
            if found.is_none() {
                eprintln!("route: no fighter matches {names:?}");
            }
            return found;
        }
        if fight.order.unwrap_or(1) >= 2 && boss.is_some() {
            return boss;
        }
        (!pool.is_empty()).then(|| pool[taken % pool.len()])
    }

    pub(crate) fn route_ladder(
        &self,
        me: usize,
        mate: Option<usize>,
        pool: &[usize],
        boss: Option<usize>,
    ) -> Option<Vec<usize>> {
        let route = load_route(&self.player_char_id())?;
        let context = self
            .route_context()
            .with_int("team_size", 1 + mate.is_some() as i64);
        let actions = match plan_route(&route, &context) {
            Ok(actions) => actions,
            Err(error) => {
                eprintln!("route {}: {error}", route.id);
                return None;
            }
        };
        let mut ladder = Vec::new();
        for action in &actions {
            match action {
                Action::Fight(fight) => {
                    if let Some(opponent) = self.route_opponent(fight, pool, boss, ladder.len()) {
                        ladder.push(opponent);
                    }
                }
                Action::Storyboard(path) => {
                    eprintln!("route {}: storyboard '{path}' is not played yet", route.id)
                }
                Action::Clear => {}
            }
        }
        ladder.retain(|&i| i != me && Some(i) != mate);
        (!ladder.is_empty()).then_some(ladder)
    }
}
