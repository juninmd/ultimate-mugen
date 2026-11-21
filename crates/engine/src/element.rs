use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Fire,
    Ice,
    Lightning,
    Water,
    Earth,
    Wind,
    Neutral,
}
