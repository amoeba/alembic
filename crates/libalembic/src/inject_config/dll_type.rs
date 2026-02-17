use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DllType {
    Alembic,
    Decal,
}

impl fmt::Display for DllType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DllType::Alembic => write!(f, "Alembic"),
            DllType::Decal => write!(f, "Decal"),
        }
    }
}
