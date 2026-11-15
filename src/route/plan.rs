use super::condition::Condition;
use super::context::Context;
use super::model::Action;

#[derive(Clone, Debug)]
pub struct Body(pub Action);

#[derive(Clone, Debug)]
pub struct Step {
    pub when: Option<Condition>,
    pub repeat: u32,
    pub main: Body,
    pub otherwise: Option<Body>,
}

#[derive(Clone, Debug)]
pub struct Route {
    pub id: String,
    pub steps: Vec<Step>,
}

pub fn plan_route(route: &Route, base: &Context) -> Result<Vec<Action>, String> {
    let mut actions: Vec<Action> = Vec::new();
    let mut fights: i64 = 0;
    for (index, step) in route.steps.iter().enumerate() {
        for _ in 0..step.repeat {
            let context = base
                .clone()
                .with_int("consecutive_wins", fights)
                .with_int("match_no", fights);
            let taken = match &step.when {
                Some(when) => when
                    .eval(&context)
                    .map_err(|e| format!("step {}: {e}", index + 1))?,
                None => true,
            };
            let chosen = if taken {
                Some(&step.main)
            } else {
                step.otherwise.as_ref()
            };
            if let Some(Body(action)) = chosen {
                if matches!(action, Action::Fight(_)) {
                    fights += 1;
                }
                actions.push(action.clone());
                if *action == Action::Clear {
                    return Ok(actions);
                }
            }
        }
    }
    Ok(actions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route::parse_route;

    const KFM: &str = r#"
[route]
id = "kfm-arcade"

[[step]]
storyboard = "chars/kfm/intro.def"

[[step]]
repeat = 6
fight = { order = 1 }

[[step]]
when = "consecutive_wins == 6"
fight = { p2 = ["SuaveDude"] }
else = { fight = { order = 3, winscreen = true } }

[[step]]
storyboard = "chars/kfm/ending.def"

[[step]]
action = "clear"
"#;

    fn base() -> Context {
        Context::default()
            .with_text("mode", "arcade")
            .with_int("stats.arcade.clear", 0)
    }

    #[test]
    fn plans_the_wiki_arcade_example() {
        let route = parse_route(KFM).unwrap();
        let actions = plan_route(&route, &base()).unwrap();
        assert_eq!(actions.len(), 1 + 6 + 1 + 1 + 1);
        assert!(matches!(actions[0], Action::Storyboard(_)));
        let secret = actions
            .iter()
            .filter(|a| matches!(a, Action::Fight(f) if f.p2.is_some()))
            .count();
        assert_eq!(secret, 1);
        assert_eq!(actions.last(), Some(&Action::Clear));
    }

    #[test]
    fn else_branch_runs_when_condition_is_false() {
        let text = KFM.replace("consecutive_wins == 6", "consecutive_wins == 99");
        let actions = plan_route(&parse_route(&text).unwrap(), &base()).unwrap();
        assert!(actions.iter().any(
            |a| matches!(a, Action::Fight(f) if f.order == Some(3) && f.winscreen == Some(true))
        ));
        assert!(!actions
            .iter()
            .any(|a| matches!(a, Action::Fight(f) if f.p2.is_some())));
    }

    #[test]
    fn clear_stops_the_plan() {
        let text = "[route]\nid = \"x\"\n[[step]]\naction = \"clear\"\n[[step]]\nfight = {}\n";
        let actions = plan_route(&parse_route(text).unwrap(), &base()).unwrap();
        assert_eq!(actions, vec![Action::Clear]);
    }

    #[test]
    fn load_errors_name_the_step() {
        let bad = |text: &str| parse_route(text).unwrap_err();
        assert!(
            bad("[route]\nid=\"x\"\n[[step]]\nwhen = \"nope > 1\"\nfight = {}\n")
                .contains("step 1")
        );
        assert!(bad("[route]\nid=\"x\"\n[[step]]\nrepeat = 0\nfight = {}\n").contains("repeat"));
        assert!(
            bad("[route]\nid=\"x\"\n[[step]]\nfight = {}\nstoryboard = \"a\"\n")
                .contains("exactly one")
        );
        assert!(bad("[route]\nid=\"x\"\n[[step]]\naction = \"win\"\n").contains("unknown action"));
        assert!(bad("[route]\nid=\"x\"\n[[step]]\nfigth = {}\n").contains("figth"));
    }
}
