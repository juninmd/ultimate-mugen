use ume_engine::diagnostics::{controller_supported, trigger_supported};

const MUGEN_CONTROLLERS: &[&str] = &[
    "afterimage", "afterimagetime", "allpalfx", "angleadd", "angledraw", "anglemul", "angleset",
    "appendtoclipboard", "assertcommand", "assertspecial", "attackdist", "attackmulset", "bgpalfx",
    "bindtoparent", "bindtoroot", "bindtotarget", "changeanim", "changeanim2", "changestate",
    "clearclipboard", "ctrlset", "defencemulset", "destroyself", "displaytoclipboard", "envcolor",
    "envshake", "explod", "explodbindtime", "fallenvshake", "forcefeedback", "gamemakeanim", "gravity",
    "helper", "hitadd", "hitby", "hitdef", "hitfalldamage", "hitfallset", "hitfallvel", "hitoverride",
    "hitvelset", "lifeadd", "lifeset", "makedust", "modifyexplod", "movehitreset", "nothitby", "null",
    "offset", "pause", "palfx", "parentvaradd", "parentvarset", "playerpush", "playsnd", "posadd",
    "posfreeze", "posset", "poweradd", "powerset", "projectile", "remappal", "removeexplod",
    "reversaldef", "screenbound", "selfstate", "sndpan", "sprpriority", "statetypeset", "stopsnd",
    "superpause", "targetbind", "targetdrop", "targetfacing", "targetlifeadd", "targetpoweradd",
    "targetstate", "targetveladd", "targetvelset", "trans", "turn", "varadd", "varrandom",
    "varrangeset", "varset", "veladd", "velmul", "velset", "victoryquote", "width",
];

const MUGEN_TRIGGERS: &[&str] = &[
    "abs", "acos", "alive", "anim", "animelemno", "animelemtime", "animexist", "animtime", "asin",
    "atan", "backedgebodydist", "backedgedist", "ceil", "cos", "ctrl", "drawgame", "exp", "facing",
    "floor", "frontedgebodydist", "frontedgedist", "gametime", "gethitvar", "hitcount", "hitfall",
    "hitover", "hitpausetime", "hitshakeover", "hitvel", "id", "ishelper", "life", "lifemax", "ln",
    "log", "loseko", "losetime", "matchno", "matchover", "max", "min", "movecontact", "moveguarded",
    "movehit", "movereversed", "numenemy", "numexplod", "numhelper", "numpartner", "numproj",
    "numprojid", "numtarget", "p2bodydist", "p2dist", "p2life", "p2stateno", "parentdist", "pi",
    "pos", "power", "powermax", "projcanceltime", "projcontact", "projguarded", "projhit", "random",
    "rootdist", "roundno", "roundsexisted", "roundstate", "screenpos", "selfanimexist", "sin",
    "stateno", "tan", "teamside", "time", "vel", "var", "fvar", "sysvar", "sysfvar", "win",
    "winko", "wintime", "winperfect", "const", "teamleader", "palno", "sprpriority", "ailevel",
    "prevstateno",
];

#[test]
fn every_mugen_controller_is_implemented() {
    let missing: Vec<&str> = MUGEN_CONTROLLERS.iter().copied().filter(|c| !controller_supported(c)).collect();
    assert!(missing.is_empty(), "MUGEN controllers without an implementation: {missing:?}");
}

#[test]
fn every_mugen_trigger_is_implemented() {
    let missing: Vec<&str> = MUGEN_TRIGGERS.iter().copied().filter(|t| !trigger_supported(t)).collect();
    assert!(missing.is_empty(), "MUGEN triggers without an implementation: {missing:?}");
}
