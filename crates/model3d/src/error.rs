use std::fmt;

#[derive(Debug)]
pub enum ModelError {
    Container(&'static str),
    Json(String),
    Missing(&'static str),
    Unsupported(&'static str),
    OutOfRange(&'static str),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::Container(m) => write!(f, "invalid glb container: {m}"),
            ModelError::Json(m) => write!(f, "invalid gltf json: {m}"),
            ModelError::Missing(m) => write!(f, "gltf is missing {m}"),
            ModelError::Unsupported(m) => write!(f, "unsupported gltf feature: {m}"),
            ModelError::OutOfRange(m) => write!(f, "gltf data out of range: {m}"),
        }
    }
}

impl std::error::Error for ModelError {}
