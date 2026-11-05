use crate::chardef::StateDef;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlagTarget {
    Character(u64),
    Global(u32),
}

pub fn normalize(raw: &str) -> String {
    raw.trim().trim_matches('"').trim().to_ascii_lowercase()
}

pub fn names_in(raw: &str) -> impl Iterator<Item = &str> + '_ {
    raw.split(',').map(|s| s.trim().trim_matches('"').trim()).filter(|s| !s.is_empty())
}

pub fn lookup(raw: &str) -> Option<FlagTarget> {
    let name = raw.trim().trim_matches('"').trim();
    asf::from_name(name).map(FlagTarget::Character).or_else(|| gsf::from_name(name).map(FlagTarget::Global))
}

pub const ASSERT_PARAMS: [&str; 5] = ["flag", "flag2", "flag3", "flag4", "flag5"];

pub fn unknown_in_states(states: &HashMap<i32, StateDef>) -> Vec<String> {
    let mut unknown: Vec<String> = states
        .values()
        .flat_map(|state| state.controllers.iter())
        .filter(|controller| controller.kind == "assertspecial")
        .flat_map(|controller| ASSERT_PARAMS.iter().filter_map(|key| controller.raw(key)))
        .flat_map(names_in)
        .filter(|name| lookup(name).is_none())
        .map(|name| name.to_ascii_lowercase())
        .collect();
    unknown.sort();
    unknown.dedup();
    unknown
}

pub fn warn_unknown(character: &str, unknown: &[String]) {
    if !unknown.is_empty() {
        eprintln!("{character}: AssertSpecial flags not recognised and ignored: {}", unknown.join(", "));
    }
}

pub mod asf {
    pub const INVISIBLE: u64 = 1 << 0;
    pub const NOAIRGUARD: u64 = 1 << 1;
    pub const NOAUTOTURN: u64 = 1 << 2;
    pub const NOCROUCHGUARD: u64 = 1 << 3;
    pub const NOJUGGLECHECK: u64 = 1 << 4;
    pub const NOKO: u64 = 1 << 5;
    pub const NOSHADOW: u64 = 1 << 6;
    pub const NOSTANDGUARD: u64 = 1 << 7;
    pub const NOWALK: u64 = 1 << 8;
    pub const UNGUARDABLE: u64 = 1 << 9;
    pub const ANIMATEHITPAUSE: u64 = 1 << 10;
    pub const ANIMFREEZE: u64 = 1 << 11;
    pub const AUTOGUARD: u64 = 1 << 12;
    pub const DRAWUNDER: u64 = 1 << 13;
    pub const NOAIBUTTONJAM: u64 = 1 << 14;
    pub const NOAICHEAT: u64 = 1 << 15;
    pub const NOAILEVEL: u64 = 1 << 16;
    pub const NOAIRJUMP: u64 = 1 << 17;
    pub const NOBRAKE: u64 = 1 << 18;
    pub const NOCOMBODISPLAY: u64 = 1 << 19;
    pub const NOCORNERPUSH: u64 = 1 << 20;
    pub const NOCROUCH: u64 = 1 << 21;
    pub const NODIZZYPOINTSDAMAGE: u64 = 1 << 22;
    pub const NOFACEDISPLAY: u64 = 1 << 23;
    pub const NOFACEP2: u64 = 1 << 24;
    pub const NOFALLCOUNT: u64 = 1 << 25;
    pub const NOFALLDEFENCEUP: u64 = 1 << 26;
    pub const NOFALLHITFLAG: u64 = 1 << 27;
    pub const NOFASTRECOVERFROMLIEDOWN: u64 = 1 << 28;
    pub const NOGETUPFROMLIEDOWN: u64 = 1 << 29;
    pub const NOGUARDBARDISPLAY: u64 = 1 << 30;
    pub const NOGUARDDAMAGE: u64 = 1 << 31;
    pub const NOGUARDKO: u64 = 1 << 32;
    pub const NOGUARDPOINTSDAMAGE: u64 = 1 << 33;
    pub const NOHARDCODEDKEYS: u64 = 1 << 34;
    pub const NOHITDAMAGE: u64 = 1 << 35;
    pub const NOINPUT: u64 = 1 << 36;
    pub const NOINTRORESET: u64 = 1 << 37;
    pub const NOJUMP: u64 = 1 << 38;
    pub const NOKOFALL: u64 = 1 << 39;
    pub const NOKOVELOCITY: u64 = 1 << 40;
    pub const NOLIFEBARACTION: u64 = 1 << 41;
    pub const NOLIFEBARDISPLAY: u64 = 1 << 42;
    pub const NOMAKEDUST: u64 = 1 << 43;
    pub const NONAMEDISPLAY: u64 = 1 << 44;
    pub const NOPOWERBARDISPLAY: u64 = 1 << 45;
    pub const NOREDLIFEDAMAGE: u64 = 1 << 46;
    pub const NOSCORE: u64 = 1 << 47;
    pub const NOSTAND: u64 = 1 << 48;
    pub const NOSTUNBARDISPLAY: u64 = 1 << 49;
    pub const NOTURNTARGET: u64 = 1 << 50;
    pub const NOWINICONDISPLAY: u64 = 1 << 51;
    pub const POSTROUNDINPUT: u64 = 1 << 52;
    pub const PROJTYPECOLLISION: u64 = 1 << 53;
    pub const RUNFIRST: u64 = 1 << 54;
    pub const RUNLAST: u64 = 1 << 55;
    pub const SIZEPUSHONLY: u64 = 1 << 56;
    pub const NODESTROYSELF: u64 = 1 << 57;
    pub const IMMOVABLE: u64 = 1 << 58;

    pub const NAMES: [(&str, u64); 59] = [
        ("invisible", INVISIBLE),
        ("noairguard", NOAIRGUARD),
        ("noautoturn", NOAUTOTURN),
        ("nocrouchguard", NOCROUCHGUARD),
        ("nojugglecheck", NOJUGGLECHECK),
        ("noko", NOKO),
        ("noshadow", NOSHADOW),
        ("nostandguard", NOSTANDGUARD),
        ("nowalk", NOWALK),
        ("unguardable", UNGUARDABLE),
        ("animatehitpause", ANIMATEHITPAUSE),
        ("animfreeze", ANIMFREEZE),
        ("autoguard", AUTOGUARD),
        ("drawunder", DRAWUNDER),
        ("noaibuttonjam", NOAIBUTTONJAM),
        ("noaicheat", NOAICHEAT),
        ("noailevel", NOAILEVEL),
        ("noairjump", NOAIRJUMP),
        ("nobrake", NOBRAKE),
        ("nocombodisplay", NOCOMBODISPLAY),
        ("nocornerpush", NOCORNERPUSH),
        ("nocrouch", NOCROUCH),
        ("nodizzypointsdamage", NODIZZYPOINTSDAMAGE),
        ("nofacedisplay", NOFACEDISPLAY),
        ("nofacep2", NOFACEP2),
        ("nofallcount", NOFALLCOUNT),
        ("nofalldefenceup", NOFALLDEFENCEUP),
        ("nofallhitflag", NOFALLHITFLAG),
        ("nofastrecoverfromliedown", NOFASTRECOVERFROMLIEDOWN),
        ("nogetupfromliedown", NOGETUPFROMLIEDOWN),
        ("noguardbardisplay", NOGUARDBARDISPLAY),
        ("noguarddamage", NOGUARDDAMAGE),
        ("noguardko", NOGUARDKO),
        ("noguardpointsdamage", NOGUARDPOINTSDAMAGE),
        ("nohardcodedkeys", NOHARDCODEDKEYS),
        ("nohitdamage", NOHITDAMAGE),
        ("noinput", NOINPUT),
        ("nointroreset", NOINTRORESET),
        ("nojump", NOJUMP),
        ("nokofall", NOKOFALL),
        ("nokovelocity", NOKOVELOCITY),
        ("nolifebaraction", NOLIFEBARACTION),
        ("nolifebardisplay", NOLIFEBARDISPLAY),
        ("nomakedust", NOMAKEDUST),
        ("nonamedisplay", NONAMEDISPLAY),
        ("nopowerbardisplay", NOPOWERBARDISPLAY),
        ("noredlifedamage", NOREDLIFEDAMAGE),
        ("noscore", NOSCORE),
        ("nostand", NOSTAND),
        ("nostunbardisplay", NOSTUNBARDISPLAY),
        ("noturntarget", NOTURNTARGET),
        ("nowinicondisplay", NOWINICONDISPLAY),
        ("postroundinput", POSTROUNDINPUT),
        ("projtypecollision", PROJTYPECOLLISION),
        ("runfirst", RUNFIRST),
        ("runlast", RUNLAST),
        ("sizepushonly", SIZEPUSHONLY),
        ("nodestroyself", NODESTROYSELF),
        ("immovable", IMMOVABLE),
    ];

    pub fn from_name(name: &str) -> Option<u64> {
        NAMES.iter().find(|(known, _)| known.eq_ignore_ascii_case(name)).map(|(_, bit)| *bit)
    }
}

pub mod gsf {
    pub const GLOBALNOKO: u32 = 1 << 0;
    pub const GLOBALNOSHADOW: u32 = 1 << 1;
    pub const INTRO: u32 = 1 << 2;
    pub const NOBARDISPLAY: u32 = 1 << 3;
    pub const NOBG: u32 = 1 << 4;
    pub const NOFG: u32 = 1 << 5;
    pub const NOKOSLOW: u32 = 1 << 6;
    pub const NOKOSND: u32 = 1 << 7;
    pub const NOMUSIC: u32 = 1 << 8;
    pub const ROUNDNOTOVER: u32 = 1 << 9;
    pub const TIMERFREEZE: u32 = 1 << 10;
    pub const CAMERAFREEZE: u32 = 1 << 11;
    pub const NOTIMEDISPLAY: u32 = 1 << 12;
    pub const ROUNDFREEZE: u32 = 1 << 13;
    pub const ROUNDNOTSKIP: u32 = 1 << 14;
    pub const SKIPFIGHTDISPLAY: u32 = 1 << 15;
    pub const SKIPKODISPLAY: u32 = 1 << 16;
    pub const SKIPROUNDDISPLAY: u32 = 1 << 17;
    pub const SKIPWINDISPLAY: u32 = 1 << 18;

    pub const NAMES: [(&str, u32); 19] = [
        ("globalnoko", GLOBALNOKO),
        ("globalnoshadow", GLOBALNOSHADOW),
        ("intro", INTRO),
        ("nobardisplay", NOBARDISPLAY),
        ("nobg", NOBG),
        ("nofg", NOFG),
        ("nokoslow", NOKOSLOW),
        ("nokosnd", NOKOSND),
        ("nomusic", NOMUSIC),
        ("roundnotover", ROUNDNOTOVER),
        ("timerfreeze", TIMERFREEZE),
        ("camerafreeze", CAMERAFREEZE),
        ("notimedisplay", NOTIMEDISPLAY),
        ("roundfreeze", ROUNDFREEZE),
        ("roundnotskip", ROUNDNOTSKIP),
        ("skipfightdisplay", SKIPFIGHTDISPLAY),
        ("skipkodisplay", SKIPKODISPLAY),
        ("skiprounddisplay", SKIPROUNDDISPLAY),
        ("skipwindisplay", SKIPWINDISPLAY),
    ];

    pub fn from_name(name: &str) -> Option<u32> {
        NAMES.iter().find(|(known, _)| known.eq_ignore_ascii_case(name)).map(|(_, bit)| *bit)
    }
}
