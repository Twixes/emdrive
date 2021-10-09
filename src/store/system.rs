use crate::construct::components::{ColumnDefinition, DataType, DataTypeRaw, TableDefinition};

pub const SYSTEM_SCHEMA_NAME: &str = "system";

pub enum SystemTable {
    Tables,
}

impl SystemTable {
    /// Array of all system tables.
    pub const ALL: [Self; 1] = [Self::Tables];

    pub fn get_definition(&self) -> TableDefinition {
        match self {
            Self::Tables => TableDefinition {
                name: "tables".into(),
                columns: vec![
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
                    ColumnDefinition {
                        name: "created_at".into(),
                        data_type: DataType {
                            raw_type: DataTypeRaw::Timestamp,
                            is_nullable: false,
                        },
                        primary_key: false,
                    },
                ],
            },
        }
    }
}
