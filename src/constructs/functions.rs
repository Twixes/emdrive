use std::{fmt, str::FromStr};

use time::OffsetDateTime;
use ulid::Ulid;
use uuid::Uuid;

use super::components::DataInstanceRaw;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Function {
    Ulid,
    Now,
}

impl Function {
    pub fn call(&self) -> DataInstanceRaw {
        match self {
            Self::Ulid => DataInstanceRaw::Uuid(Uuid::from(Ulid::new())),
            Self::Now => DataInstanceRaw::Timestamp(OffsetDateTime::now_utc()),
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "keyword `{}`",
            match self {
                Self::Ulid => "ULID",
                Self::Now => "NOW",
            }
        )
    }
}

impl FromStr for Function {
    type Err = String;

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        match candidate.to_lowercase().as_str() {
            "ulid" => Ok(Self::Ulid),
            "now" => Ok(Self::Now),
            _ => Err(format!(
                "`{}` does not refer to a supported function",
                candidate
            )),
        }
    }
}
