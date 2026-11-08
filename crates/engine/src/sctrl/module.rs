use crate::chardef::Controller;
use crate::matchs::Match;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleStatus {
    Supported,
    RejectedByDesign,
    Planned,
}

pub const MODULE_CONTROLLERS: &[(&str, ModuleStatus)] = &[
    ("matchrestart", ModuleStatus::Supported),
    ("loadfile", ModuleStatus::RejectedByDesign),
    ("savefile", ModuleStatus::RejectedByDesign),
    ("storyboard", ModuleStatus::Planned),
    ("removestoryboard", ModuleStatus::Planned),
    ("dialogue", ModuleStatus::Planned),
    ("savestate", ModuleStatus::Planned),
    ("loadstate", ModuleStatus::Planned),
    ("lifebaraction", ModuleStatus::Planned),
    ("changemovelist", ModuleStatus::Planned),
    ("shaderset", ModuleStatus::RejectedByDesign),
];

pub fn module_status(kind: &str) -> Option<ModuleStatus> {
    MODULE_CONTROLLERS
        .iter()
        .find(|(name, _)| *name == kind)
        .map(|(_, status)| *status)
}

pub fn rejected_by_design() -> impl Iterator<Item = &'static str> {
    MODULE_CONTROLLERS
        .iter()
        .filter(|(_, s)| *s == ModuleStatus::RejectedByDesign)
        .map(|(name, _)| *name)
}

pub fn run(m: &mut Match, i: usize, sc: &Controller) -> bool {
    let Some(status) = module_status(sc.kind.as_str()) else {
        return false;
    };
    if sc.kind == "matchrestart" {
        m.restart_pending = true;
        return true;
    }
    crate::diagnostics::note_unsupported(crate::diagnostics::Category::Controller, &sc.kind);
    let reason = match status {
        ModuleStatus::RejectedByDesign => "rejected by design, ignored",
        _ => "not implemented yet, ignored",
    };
    let message = format!("{} ({}): {}", sc.kind, m.fighters[i].def.name, reason);
    m.console.warn_once(&sc.kind, &message);
    true
}
