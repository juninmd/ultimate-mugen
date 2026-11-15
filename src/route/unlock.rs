use super::condition::Condition;
use super::context::Context;

#[derive(Clone, Debug)]
pub struct Rule {
    pub target: String,
    pub when: Condition,
}

#[derive(Clone, Debug, Default)]
pub struct UnlockRules {
    pub unlock: Vec<Rule>,
    pub hide: Vec<Rule>,
}

impl UnlockRules {
    pub fn allows(&self, target: &str, context: &Context) -> bool {
        let unlocked = self
            .unlock
            .iter()
            .filter(|r| r.target == target)
            .all(|r| r.when.eval(context).unwrap_or(false));
        let hidden = self
            .hide
            .iter()
            .filter(|r| r.target == target)
            .any(|r| r.when.eval(context).unwrap_or(false));
        unlocked && !hidden
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route::parse_unlocks;

    const RULES: &str = r#"
[[unlock]]
target = "char:oboro"
when = "stats.arcade.clear >= 1"

[[unlock]]
target = "stage:dojo"
when = "stats.arcade.clear >= 3 || netplay"

[[hide]]
target = "char:oboro"
when = "mode == \"bossrush\""
"#;

    fn ctx(clears: i64, mode: &str, netplay: i64) -> Context {
        Context::default()
            .with_int("stats.arcade.clear", clears)
            .with_text("mode", mode)
            .with_int("netplay", netplay)
    }

    #[test]
    fn locked_until_condition_holds() {
        let rules = parse_unlocks(RULES).unwrap();
        assert!(!rules.allows("char:oboro", &ctx(0, "arcade", 0)));
        assert!(rules.allows("char:oboro", &ctx(1, "arcade", 0)));
        assert!(!rules.allows("stage:dojo", &ctx(2, "arcade", 0)));
        assert!(rules.allows("stage:dojo", &ctx(0, "arcade", 1)));
    }

    #[test]
    fn hide_overrides_unlock_and_unlisted_targets_are_open() {
        let rules = parse_unlocks(RULES).unwrap();
        assert!(!rules.allows("char:oboro", &ctx(5, "bossrush", 0)));
        assert!(rules.allows("char:kaito", &ctx(0, "arcade", 0)));
    }

    #[test]
    fn bad_rules_are_rejected() {
        assert!(parse_unlocks("[[unlock]]\ntarget = \"oboro\"\nwhen = \"true\"\n").is_err());
        assert!(parse_unlocks("[[hide]]\ntarget = \"item:x\"\nwhen = \"true\"\n").is_err());
        assert!(parse_unlocks("[[unlock]]\ntarget = \"char:a\"\nwhen = \"zzz\"\n").is_err());
    }
}
