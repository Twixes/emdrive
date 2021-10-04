use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataTypeRaw {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Timestamp,
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
            "string" => Ok(Self::String),
            _ => Err(format!(
                "`{}` does not refer to a supported type",
                candidate
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub primary_key: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TableDefinition {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}
