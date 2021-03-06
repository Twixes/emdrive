use crate::sql::ValidationError;

use super::components::*;

#[derive(Debug, PartialEq, Eq)]
pub struct CreateTableStatement {
    pub table: TableDefinition,
    pub if_not_exists: bool,
}

impl Validatable for CreateTableStatement {
    fn validate(&self) -> Result<(), ValidationError> {
        self.table.validate()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InsertStatement {
    pub table_name: String,
    pub column_names: Vec<String>,
    pub values: Vec<DataInstance>,
}

impl Validatable for InsertStatement {
    fn validate(&self) -> Result<(), ValidationError> {
        Ok(()) // TODO: Add checks
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SelectColumn {
    All,
    Identifier(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct SelectStatement {
    pub columns: Vec<SelectColumn>,
    /// String means table name
    pub source: String,
    pub where_clause: Option<Expression>,
}

impl Validatable for SelectStatement {
    fn validate(&self) -> Result<(), ValidationError> {
        Ok(()) // TODO: Add checks
    }
}
