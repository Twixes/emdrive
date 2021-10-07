use super::components::*;

#[derive(Debug, PartialEq, Eq)]
pub struct CreateTableStatement {
    pub table: TableDefinition,
    pub if_not_exists: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InsertStatement {
    pub table_name: String,
    pub column_names: Vec<String>,
    pub values: Vec<DataInstance>,
}
