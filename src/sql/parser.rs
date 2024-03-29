use super::expects::*;
use super::tokenizer::*;
use crate::constructs::components::Validatable;
use crate::constructs::statements::SelectStatement;
use crate::constructs::statements::{CreateTableStatement, InsertStatement};
use crate::sql::errors::*;

pub fn parse_statement(input: &str) -> Result<Statement, SyntaxError> {
    let tokens = tokenize_statement(input);
    let ExpectOk {
        rest,
        outcome: found_token_first,
        ..
    } = expect_next_token(
        &tokens,
        &format!("{} or {}", Keyword::Create, Keyword::Insert),
    )?;
    match found_token_first {
        // CREATE
        Token {
            value: TokenValue::Const(Keyword::Create),
            ..
        } => {
            let ExpectOk {
                rest,
                outcome: found_token_second,
                ..
            } = expect_next_token(rest, &Keyword::Table.to_string())?;
            match found_token_second {
                // CREATE TABLE
                Token {
                    value: TokenValue::Const(Keyword::Table),
                    ..
                } => Ok(Statement::CreateTable(consume_all(
                    rest,
                    expect_create_table,
                )?)),
                // CREATE ???
                wrong_token => Err(SyntaxError(format!(
                    "Expected {}, instead found {}.",
                    Keyword::Table,
                    wrong_token
                ))),
            }
        }
        // INSERT
        Token {
            value: TokenValue::Const(Keyword::Insert),
            ..
        } => Ok(Statement::Insert(consume_all(rest, expect_insert)?)),
        // SELECT
        Token {
            value: TokenValue::Const(Keyword::Select),
            ..
        } => Ok(Statement::Select(consume_all(rest, expect_select)?)),
        // Something else
        wrong_token => Err(SyntaxError(format!(
            "Expected {} or {}, instead found {}.",
            Keyword::Create,
            Keyword::Insert,
            wrong_token
        ))),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    CreateTable(CreateTableStatement),
    Insert(InsertStatement),
    Select(SelectStatement),
}

impl Validatable for Statement {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Statement::CreateTable(create_table) => create_table.validate(),
            Statement::Insert(insert) => insert.validate(),
            Statement::Select(select) => select.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::constructs::{
        components::{
            ColumnDefinition, DataDefinition, DataInstance, DataInstanceRaw, DataType, DataTypeRaw,
            Expression, TableDefinition,
        },
        functions::Function,
        statements::SelectColumn,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parsing_works_with_create_table() {
        const STATEMENT: &str = "CREATE TABLE IF NOT EXISTS test (
            id STRING PRIMARY KEY,
            server_id nullable(UINT64),
            hash UINT128 DEFAULT 666,
            sent_at TIMESTAMP DEFAULT NOW()
        );";

        let detected_statement = parse_statement(STATEMENT).unwrap();

        assert_eq!(
            detected_statement,
            Statement::CreateTable(CreateTableStatement {
                table: TableDefinition::new(
                    "test".to_string(),
                    vec![
                        ColumnDefinition {
                            name: "id".to_string(),
                            data_type: DataType {
                                raw_type: DataTypeRaw::String,
                                is_nullable: false
                            },
                            primary_key: true,
                            default: None,
                        },
                        ColumnDefinition {
                            name: "server_id".to_string(),
                            data_type: DataType {
                                raw_type: DataTypeRaw::UInt64,
                                is_nullable: true
                            },
                            primary_key: false,
                            default: None,
                        },
                        ColumnDefinition {
                            name: "hash".to_string(),
                            data_type: DataType {
                                raw_type: DataTypeRaw::UInt128,
                                is_nullable: false
                            },
                            primary_key: false,
                            default: Some(DataDefinition::Const(DataInstance::Direct(
                                // TODO: Infer number size from context
                                DataInstanceRaw::UInt32(666)
                            ))),
                        },
                        ColumnDefinition {
                            name: "sent_at".to_string(),
                            data_type: DataType {
                                raw_type: DataTypeRaw::Timestamp,
                                is_nullable: false
                            },
                            primary_key: false,
                            default: Some(DataDefinition::FunctionCall(Function::Now)),
                        },
                    ]
                ),
                if_not_exists: true
            })
        )
    }

    #[test]
    fn parsing_works_with_insert() {
        const STATEMENT: &str = "INSERT INTO xyz (foo, bar)
        VALUES (1815, 'Waterloo');";

        let detected_statement = parse_statement(STATEMENT).unwrap();

        assert_eq!(
            detected_statement,
            Statement::Insert(InsertStatement {
                table_name: "xyz".to_string(),
                column_names: vec!["foo".to_string(), "bar".to_string(),],
                values: vec![
                    DataInstance::Direct(DataInstanceRaw::UInt32(1815)),
                    DataInstance::Direct(DataInstanceRaw::String("Waterloo".into())),
                ]
            })
        )
    }

    #[test]
    fn parsing_works_with_select() {
        const STATEMENT: &str = "SELECT *, foo FROM xyz WHERE foo = 'bar';";

        let detected_statement = parse_statement(STATEMENT).unwrap();

        assert_eq!(
            detected_statement,
            Statement::Select(SelectStatement {
                columns: vec![
                    SelectColumn::All,
                    SelectColumn::Identifier("foo".to_string()),
                ],
                source: "xyz".to_string(),
                where_clause: Some(Expression::Equal(
                    Box::new(Expression::Atom(DataDefinition::Identifier(
                        "foo".to_string()
                    ))),
                    Box::new(Expression::Atom(DataDefinition::Const(
                        DataInstance::Direct(DataInstanceRaw::String("bar".into()))
                    )))
                ))
            })
        )
    }
}
