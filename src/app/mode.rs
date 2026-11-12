#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Arcade,
    Story,
    Survival,
    Cpu,
    Versus,
    Training,
    Watch,
    TeamSeq,
    TeamSimul,
    TeamTag,
}
impl Mode {
    pub(super) fn label(self) -> &'static str {
        crate::i18n::t(match self {
            Mode::Arcade => "mode.arcade",
            Mode::Story => return crate::i18n::tr("HISTÓRIA", "STORY"),
            Mode::Survival => "mode.survival",
            Mode::Cpu => "mode.cpu",
            Mode::Versus => "mode.versus",
            Mode::Training => "mode.training",
            Mode::Watch => "mode.watch",
            Mode::TeamSeq => "mode.team_seq",
            Mode::TeamSimul => "mode.team_simul",
            Mode::TeamTag => "mode.team_tag",
        })
    }
    /// Team size (1 for the regular modes).
    pub fn is_team(self) -> bool {
        matches!(self, Mode::TeamSeq | Mode::TeamSimul | Mode::TeamTag)
    }
    /// Stable key used by replay files.
    pub fn key(self) -> &'static str {
        match self {
            Mode::Arcade => "Arcade",
            Mode::Story => "Story",
            Mode::Survival => "Survival",
            Mode::Cpu => "Cpu",
            Mode::Versus => "Versus",
            Mode::Training => "Training",
            Mode::Watch => "Watch",
            Mode::TeamSeq => "TeamSeq",
            Mode::TeamSimul => "TeamSimul",
            Mode::TeamTag => "TeamTag",
        }
    }
    pub fn from_key(key: &str) -> Mode {
        match key {
            "Arcade" => Mode::Arcade,
            "Story" => Mode::Story,
            "Survival" => Mode::Survival,
            "Versus" => Mode::Versus,
            "Training" => Mode::Training,
            "Watch" => Mode::Watch,
            "TeamSeq" => Mode::TeamSeq,
            "TeamSimul" => Mode::TeamSimul,
            "TeamTag" => Mode::TeamTag,
            _ => Mode::Cpu,
        }
    }
}
