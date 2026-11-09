use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FightSpec {
    pub order: Option<u32>,
    pub p2: Option<Vec<String>>,
    pub stage: Option<String>,
    pub winscreen: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Fight(FightSpec),
    Storyboard(String),
    Clear,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBody {
    pub fight: Option<FightSpec>,
    pub storyboard: Option<String>,
    pub action: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStep {
    pub when: Option<String>,
    pub repeat: Option<u32>,
    pub fight: Option<FightSpec>,
    pub storyboard: Option<String>,
    pub action: Option<String>,
    #[serde(rename = "else")]
    pub otherwise: Option<RawBody>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHeader {
    pub id: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRoute {
    pub route: RawHeader,
    #[serde(default)]
    pub step: Vec<RawStep>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRule {
    pub target: String,
    pub when: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawUnlocks {
    #[serde(default)]
    pub unlock: Vec<RawRule>,
    #[serde(default)]
    pub hide: Vec<RawRule>,
}
