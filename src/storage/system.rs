use crate::constructs::components::{ColumnDefinition, DataType, DataTypeRaw, TableDefinition};

pub const SYSTEM_SCHEMA_NAME: &str = "system";

pub enum SystemTable {
    Tables,
    Columns,
}

impl SystemTable {
    /// Array of all system tables.
    pub const ALL: [Self; 2] = [Self::Tables, Self::Columns];

    pub fn get_definition(&self) -> TableDefinition {
        match self {
            Self::Tables => TableDefinition::new(
                "tables".into(),
                vec![
                    ColumnDefinition {
                        name: "id".into(),
                        data_type: DataType {
                            raw_type: DataTypeRaw::Uuid,
                            is_nullable: false,
                        },
                        primary_key: true,
                    },
                    ColumnDefinition {
                        name: "table_name".into(),
                        data_type: DataType {
                            raw_type: DataTypeRaw::String,
                            is_nullable: false,
                        },
                        primary_key: false,
                    },
                ],
            ),
            Self::Columns => TableDefinition::new(
                "columns".into(),
                vec![
                    ColumnDefinition {
                        name: "id".into(),
                        data_type: DataType {
                            raw_type: DataTypeRaw::Uuid,
                            is_nullable: false,
                        },
                        primary_key: true,
                    },
                    ColumnDefinition {
                        name: "table_id".into(),
                        data_type: DataType {
                            raw_type: DataTypeRaw::String,
                            is_nullable: false,
                        },
                        primary_key: false,
                    },
                    ColumnDefinition {
                        name: "raw_type".into(),
                        data_type: DataType {
                            raw_type: DataTypeRaw::String,
                            is_nullable: false,
                        },
                        primary_key: false,
                    },
                    ColumnDefinition {
                        name: "is_nullable".into(),
                        data_type: DataType {
                            raw_type: DataTypeRaw::Bool,
                            is_nullable: false,
                        },
                        primary_key: false,
                    },
                ],
            ),
        }
    }
}
