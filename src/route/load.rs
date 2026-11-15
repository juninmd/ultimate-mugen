use super::condition::Condition;
use super::model::{Action, RawBody, RawRoute, RawUnlocks};
use super::plan::{Body, Route, Step};
use super::unlock::{Rule, UnlockRules};

pub const MAX_REPEAT: u32 = 99;
const TARGET_KINDS: [&str; 2] = ["char", "stage"];

fn body(raw: RawBody, place: &str) -> Result<Body, String> {
    let chosen = [
        raw.fight.is_some(),
        raw.storyboard.is_some(),
        raw.action.is_some(),
    ];
    if chosen.iter().filter(|&&c| c).count() != 1 {
        return Err(format!(
            "{place}: set exactly one of fight, storyboard, action"
        ));
    }
    if let Some(fight) = raw.fight {
        return Ok(Body(Action::Fight(fight)));
    }
    if let Some(path) = raw.storyboard {
        return Ok(Body(Action::Storyboard(path)));
    }
    match raw.action.as_deref() {
        Some("clear") => Ok(Body(Action::Clear)),
        Some(other) => Err(format!("{place}: unknown action '{other}' (known: clear)")),
        None => unreachable!(),
    }
}

fn condition(source: &str, place: &str) -> Result<Condition, String> {
    Condition::parse(source).map_err(|e| format!("{place}: {e}"))
}

pub fn parse_route(text: &str) -> Result<Route, String> {
    let raw: RawRoute = toml::from_str(text).map_err(|e| e.to_string())?;
    let mut steps = Vec::new();
    for (index, step) in raw.step.into_iter().enumerate() {
        let place = format!("step {}", index + 1);
        let repeat = step.repeat.unwrap_or(1);
        if repeat == 0 || repeat > MAX_REPEAT {
            return Err(format!(
                "{place}: repeat must be between 1 and {MAX_REPEAT}"
            ));
        }
        let when = step
            .when
            .as_deref()
            .map(|w| condition(w, &place))
            .transpose()?;
        let main = body(
            RawBody {
                fight: step.fight,
                storyboard: step.storyboard,
                action: step.action,
            },
            &place,
        )?;
        let otherwise = step
            .otherwise
            .map(|o| body(o, &format!("{place} else")))
            .transpose()?;
        steps.push(Step {
            when,
            repeat,
            main,
            otherwise,
        });
    }
    Ok(Route {
        id: raw.route.id,
        steps,
    })
}

fn rules(raw: Vec<super::model::RawRule>, kind: &str) -> Result<Vec<Rule>, String> {
    raw.into_iter()
        .enumerate()
        .map(|(index, rule)| {
            let place = format!("{kind} {}", index + 1);
            let (target_kind, id) = rule
                .target
                .split_once(':')
                .ok_or_else(|| format!("{place}: target must look like char:<id> or stage:<id>"))?;
            if !TARGET_KINDS.contains(&target_kind) || id.is_empty() {
                return Err(format!(
                    "{place}: target must look like char:<id> or stage:<id>"
                ));
            }
            Ok(Rule {
                target: rule.target.clone(),
                when: condition(&rule.when, &place)?,
            })
        })
        .collect()
}

pub fn parse_unlocks(text: &str) -> Result<UnlockRules, String> {
    let raw: RawUnlocks = toml::from_str(text).map_err(|e| e.to_string())?;
    Ok(UnlockRules {
        unlock: rules(raw.unlock, "unlock")?,
        hide: rules(raw.hide, "hide")?,
    })
}
