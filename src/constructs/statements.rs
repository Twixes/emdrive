use crate::sql::StatementValidationError;

use super::components::*;

#[derive(Debug, PartialEq, Eq)]
pub struct CreateTableStatement {
    pub table: TableDefinition,
    pub if_not_exists: bool,
}

impl Validatable for CreateTableStatement {
    fn validate(&self) -> Result<(), StatementValidationError> {
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
    fn validate(&self) -> Result<(), StatementValidationError> {
        Ok(()) // TODO: Add checks
    }
}
