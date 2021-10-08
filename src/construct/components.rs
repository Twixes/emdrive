use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataTypeRaw {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Timestamp,
    Uuid,
    String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DataInstance {
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    Timestamp(u64),
    Uuid(u128),
    String(String),
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

trait Validatable {
    /// Make sure that this definition (self) actually makes sense.
    fn validate(&self) -> Result<(), String>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub primary_key: bool,
}

impl Validatable for ColumnDefinition {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("A column must have a name".into());
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TableDefinition {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

impl Validatable for TableDefinition {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("A table must have a name".into());
        }
        if self.columns.is_empty() {
            return Err("A table must have at least one column".into());
        }
        let mut primary_key_count = 0;
        for (column_index, column) in self.columns.iter().enumerate() {
            if column.primary_key {
                primary_key_count += 1;
            }
            if let Err(column_error) = column.validate() {
                return Err(format!(
                    "Problem at column {}: {}",
                    column_index + 1,
                    column_error
                ));
            }
        }
        if primary_key_count != 1 {
            return Err(format!(
                "A table must have exactly 1 PRIMARY KEY column, not {}",
                primary_key_count
            ));
        }
        Ok(())
    }
}
