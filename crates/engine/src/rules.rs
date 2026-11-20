use crate::chardef::{CharDef, Source};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FighterRules {
    pub combo_damage_scaling: bool,
    pub hitdef_corner_push: bool,
    pub engine_corner_push: bool,
    pub clears_hitdef_on_strike: bool,
    pub legacy_awakening: bool,
    pub own_awakening_anim: bool,
    pub perfect_guard: bool,
    pub script_landing: bool,
    pub exact_anim_time: bool,
}

impl FighterRules {
    pub const MUGEN: FighterRules = FighterRules {
        combo_damage_scaling: false,
        hitdef_corner_push: true,
        engine_corner_push: false,
        clears_hitdef_on_strike: true,
        legacy_awakening: true,
        own_awakening_anim: false,
        perfect_guard: false,
        script_landing: true,
        exact_anim_time: true,
    };

    pub const NATIVE: FighterRules = FighterRules {
        combo_damage_scaling: true,
        hitdef_corner_push: false,
        engine_corner_push: true,
        clears_hitdef_on_strike: false,
        legacy_awakening: false,
        own_awakening_anim: true,
        perfect_guard: true,
        script_landing: false,
        exact_anim_time: false,
    };
}

impl Source {
    pub const fn rules(self) -> FighterRules {
        match self {
            Source::Mugen => FighterRules::MUGEN,
            Source::Native => FighterRules::NATIVE,
        }
    }
}

impl CharDef {
    pub fn rules(&self) -> FighterRules {
        self.source.rules()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mugen_and_native_rules_never_overlap_on_corner_push() {
        assert!(FighterRules::MUGEN.hitdef_corner_push && !FighterRules::MUGEN.engine_corner_push);
        assert!(FighterRules::NATIVE.engine_corner_push && !FighterRules::NATIVE.hitdef_corner_push);
    }
}
