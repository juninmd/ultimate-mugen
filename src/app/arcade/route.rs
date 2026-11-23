use crate::app::{App, Mode};
use crate::gfx::get_time;
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

    pub(crate) fn route_gate(&self) -> impl Fn(usize) -> bool + '_ {
        let context = self.route_context();
        move |char_index| self.route_allows_in(char_index, &context)
    }

    fn route_allows_in(&self, char_index: usize, context: &Context) -> bool {
        let id = self.reg.chars[char_index]
            .id
            .strip_prefix("umf:")
            .unwrap_or(&self.reg.chars[char_index].id);
        unlock_rules().allows(&format!("char:{id}"), context)
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

    fn route_stage(&self, wanted: &str, fight_index: usize) -> Option<usize> {
        let stage_count = self.reg.stages.len();
        if stage_count == 0 {
            return None;
        }
        let random_stage = if wanted.trim().eq_ignore_ascii_case("random") {
            ((get_time() * 7.0) as usize).wrapping_add(fight_index) % stage_count
        } else {
            0
        };
        stage_selection(
            wanted,
            random_stage,
            self.reg
                .stages
                .iter()
                .map(|stage| (stage.id.as_str(), stage.name.as_str())),
        )
    }

    pub(crate) fn route_ladder(
        &self,
        me: usize,
        mate: Option<usize>,
        pool: &[usize],
        boss: Option<usize>,
    ) -> Option<(Vec<usize>, Vec<Option<usize>>)> {
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
        let mut stages = Vec::new();
        let mut planned_fights = 0;
        for action in &actions {
            match action {
                Action::Fight(fight) => {
                    if let Some(opponent) = self.route_opponent(fight, pool, boss, planned_fights) {
                        planned_fights += 1;
                        if opponent == me || Some(opponent) == mate {
                            continue;
                        }
                        ladder.push(opponent);
                        let stage = fight.stage.as_deref().and_then(|name| {
                            let stage = self.route_stage(name, planned_fights - 1);
                            if stage.is_none() {
                                eprintln!("route {}: no stage matches '{name}'", route.id);
                            }
                            stage
                        });
                        stages.push(stage);
                    }
                }
                Action::Storyboard(path) => {
                    eprintln!("route {}: storyboard '{path}' is not played yet", route.id)
                }
                Action::Clear => {}
            }
        }
        (!ladder.is_empty()).then_some((ladder, stages))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn route_stage_matches_ids_names_and_random_selection() {
        let stages = [("native:city", "City at Night"), ("mugen:dojo", "Dojo")];
        let find = |name| super::stage_selection(name, 1, stages.iter().copied());
        assert_eq!(find("city"), Some(0));
        assert_eq!(find("native:city"), Some(0));
        assert_eq!(find("City at Night"), Some(0));
        assert_eq!(find("DOJO"), Some(1));
        assert_eq!(find("random"), Some(1));
        assert_eq!(find(" random "), Some(1));
        assert_eq!(find("missing"), None);
    }
}

fn stage_selection<'a>(
    wanted: &str,
    random_stage: usize,
    stages: impl Iterator<Item = (&'a str, &'a str)>,
) -> Option<usize> {
    if wanted.trim().eq_ignore_ascii_case("random") {
        return Some(random_stage);
    }
    let wanted = wanted.trim();
    stages
        .enumerate()
        .find(|(_, (id, name))| {
            if id.eq_ignore_ascii_case(wanted) {
                return true;
            }
            let id = id.strip_prefix("native:").unwrap_or(id);
            let id = id.strip_prefix("mugen:").unwrap_or(id);
            id.eq_ignore_ascii_case(wanted) || name.eq_ignore_ascii_case(wanted)
        })
        .map(|(index, _)| index)
}
