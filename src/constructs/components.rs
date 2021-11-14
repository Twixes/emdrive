use serde::Serialize;
use std::{collections::HashSet, str::FromStr};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::sql::ValidationError;

use super::functions::Function;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataTypeRaw {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Bool,
    Timestamp,
    Uuid,
    String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataType {
    pub raw_type: DataTypeRaw,
    pub is_nullable: bool,
}

impl FromStr for DataTypeRaw {
    type Err = String;

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        match candidate.to_lowercase().as_str() {
            "uint8" => Ok(Self::UInt8),
            "uint16" => Ok(Self::UInt16),
            "uint32" => Ok(Self::UInt32),
            "uint64" => Ok(Self::UInt64),
            "uint128" => Ok(Self::UInt128),
            "bool" => Ok(Self::Bool),
            "timestamp" => Ok(Self::Timestamp),
            "uuid" => Ok(Self::Uuid),
            "string" => Ok(Self::String),
            _ => Err(format!(
                "`{}` does not refer to a supported type",
                candidate
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(untagged)]
pub enum DataInstanceRaw {
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    Bool(bool),
    Timestamp(OffsetDateTime),
    Uuid(Uuid),
    String(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(untagged)]
pub enum DataInstance {
    Direct(DataInstanceRaw),
    Nullable(DataInstanceRaw),
    Null,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataDefinition {
    Const(DataInstance),
    FunctionCall(Function),
}

pub trait Validatable {
    /// Make sure that this definition (self) actually makes sense.
    fn validate(&self) -> Result<(), ValidationError>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub primary_key: bool,
    pub default: Option<DataDefinition>,
}

impl Validatable for ColumnDefinition {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError("A column must have a name".into()));
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TableDefinition {
    // Table name.
    pub name: String,
    // Column definitions.
    pub columns: Vec<ColumnDefinition>,
}

impl TableDefinition {
    pub fn new(name: String, columns: Vec<ColumnDefinition>) -> Self {
        TableDefinition { name, columns }
    }

    pub fn get_primary_key(&self) -> &ColumnDefinition {
        self.columns
            .iter()
            .find(|column| column.primary_key)
            .expect("A table must have a PRIMARY KEY column")
    }
}

impl Validatable for TableDefinition {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError("A table must have a name".into()));
        }
        if self.columns.is_empty() {
            return Err(ValidationError(
                "A table must have at least one column".into(),
            ));
        }
        let mut primary_key_count = 0;
        let mut column_names: HashSet<String> = HashSet::new();
        for (column_index, column) in self.columns.iter().enumerate() {
            if column_names.contains(&column.name) {
                return Err(ValidationError(format!(
                    "There is more than one column with name `{}` in table definition",
                    column.name
                )));
            }
            column_names.insert(column.name.clone());
            if column.primary_key {
                primary_key_count += 1;
            }
            if let Err(column_error) = column.validate() {
                return Err(ValidationError(format!(
                    "Problem at column {}: {}",
                    column_index + 1,
                    column_error
                )));
            }
        }
        if primary_key_count != 1 {
            return Err(ValidationError(format!(
                "A table must have exactly 1 PRIMARY KEY column, not {}",
                primary_key_count
            )));
        }
        Ok(())
    }
}
