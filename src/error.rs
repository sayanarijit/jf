pub use serde_json as json;
pub use serde_yaml as yaml;
use std::fmt::Display;
use std::io;

#[derive(Debug)]
pub enum Error {
    Json(json::Error),
    Yaml(yaml::Error),
    Jf(String),
    Io(io::Error),
}

impl Error {
    pub fn returncode(&self) -> i32 {
        match self {
            Self::Jf(_) => 1,
            Self::Json(_) => 2,
            Self::Yaml(_) => 3,
            Self::Io(_) => 4,
        }
    }
}

impl From<yaml::Error> for Error {
    fn from(v: yaml::Error) -> Self {
        Self::Yaml(v)
    }
}

impl From<json::Error> for Error {
    fn from(v: json::Error) -> Self {
        Self::Json(v)
    }
}

impl From<&str> for Error {
    fn from(v: &str) -> Self {
        Self::Jf(v.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Self::Io(v)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(e) => write!(f, "json: {e}"),
            Self::Yaml(e) => write!(f, "yaml: {e}"),
            Self::Jf(e) => write!(f, "jf: {e}"),
            Self::Io(e) => write!(f, "io: {e}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
