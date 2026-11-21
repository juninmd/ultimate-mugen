use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Severity {
    Critical,
    Major,
    Minor,
    Info,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::Major => "major",
            Severity::Minor => "minor",
            Severity::Info => "info",
        }
    }

    pub fn from_label(text: &str) -> Option<Severity> {
        match text {
            "critical" => Some(Severity::Critical),
            "major" => Some(Severity::Major),
            "minor" => Some(Severity::Minor),
            "info" => Some(Severity::Info),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Issue {
    pub severity: Severity,
    pub category: &'static str,
    pub code: &'static str,
    pub subject: String,
    pub detail: String,
}

impl Issue {
    pub fn new(severity: Severity, category: &'static str, code: &'static str, subject: impl Into<String>, detail: impl Into<String>) -> Self {
        Issue { severity, category, code, subject: subject.into(), detail: detail.into() }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FightResult {
    pub stage: String,
    pub opponent: String,
    pub ticks: u32,
    pub hits: u32,
    pub damage: f32,
    pub distinct_states: usize,
    pub stuck_ticks: i32,
    pub tick_us: f64,
    pub verdict: &'static str,
}

#[derive(Clone, Debug, Default)]
pub struct CharResult {
    pub def: String,
    pub name: String,
    pub author: String,
    pub issues: Vec<Issue>,
    pub fights: Vec<FightResult>,
    pub load_ms: u128,
    pub anims: usize,
    pub states: usize,
    pub sprites: usize,
    pub sounds: usize,
    pub sprite_bytes: u64,
    pub sprite_pixels: u64,
}

#[derive(Clone, Debug, Default)]
pub struct StageResult {
    pub def: String,
    pub name: String,
    pub issues: Vec<Issue>,
    pub fights: Vec<FightResult>,
    pub load_ms: u128,
    pub layers: usize,
    pub sprites: usize,
    pub sprite_bytes: u64,
}

#[derive(Default)]
pub struct Report {
    pub chars: Vec<CharResult>,
    pub stages: Vec<StageResult>,
}

impl Report {
    pub fn issues(&self) -> impl Iterator<Item = &Issue> {
        self.chars.iter().flat_map(|c| c.issues.iter()).chain(self.stages.iter().flat_map(|s| s.issues.iter()))
    }

    pub fn count(&self, severity: Severity) -> usize {
        self.issues().filter(|i| i.severity == severity).count()
    }

    pub fn worst(&self) -> Option<Severity> {
        self.issues().map(|i| i.severity).min()
    }
}

pub struct Options {
    pub chars_root: PathBuf,
    pub stages_root: PathBuf,
    pub out: PathBuf,
    pub chars: Option<usize>,
    pub stages: Option<usize>,
    pub fights: usize,
    pub stage_fights: usize,
    pub filter: Option<String>,
    pub stage_filter: Option<String>,
    pub opponent: Option<String>,
    pub skip_fights: bool,
    pub quiet: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            chars_root: PathBuf::from("mugen/chars"),
            stages_root: PathBuf::from("mugen/stages"),
            out: PathBuf::from("docs/evidence/homologation"),
            chars: None,
            stages: None,
            fights: 2,
            stage_fights: 1,
            filter: None,
            stage_filter: None,
            opponent: None,
            skip_fights: false,
            quiet: false,
        }
    }
}

pub struct Job {
    pub def: String,
    pub dir: String,
}

impl Clone for Job {
    fn clone(&self) -> Self {
        Job { def: self.def.clone(), dir: self.dir.clone() }
    }
}
