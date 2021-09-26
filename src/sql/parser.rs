use super::errors::*;
use super::expects::*;
use super::tokenizer::*;

fn parse_create_table(tokens: &[Token]) -> Result<CreateTableStatement, SyntaxError> {
    let (if_not_exists, rest) = match expect_token_values_sequence(
        tokens,
        &[
            TokenValue::Const(Keyword::If),
            TokenValue::Const(Keyword::Not),
            TokenValue::Const(Keyword::Exists),
        ],
    ) {
        Ok(ExpectOk { rest, .. }) => (true, rest),
        Err(_) => (false, tokens),
    };
    let ExpectOk {
        outcome: table,
        rest,
        ..
    } = expect_table_definition(rest)?;
    expect_end_of_statement(rest)?;
    Ok(CreateTableStatement {
        table,
        if_not_exists,
    })
}

fn parse_create(tokens: &[Token]) -> Result<Statement, SyntaxError> {
    match tokens.first() {
        Some(Token {
            value: TokenValue::Const(Keyword::Table),
            ..
        }) => Ok(Statement::CreateTable(parse_create_table(&tokens[1..])?)),
        Some(wrong_token) => Err(SyntaxError(format!(
            "Expected {}, instead found `{}`.",
            Keyword::Table,
            wrong_token
        ))),
        None => Err(SyntaxError(format!(
            "Expected {}, instead found end of statement.",
            Keyword::Table
        ))),
    }
}

pub fn parse_statement(input: &str) -> Result<Statement, SyntaxError> {
    let tokens = tokenize_statement(input);
    match tokens.first() {
        Some(Token {
            value: TokenValue::Const(Keyword::Create),
            ..
        }) => parse_create(&tokens[1..]),
        Some(wrong_token) => Err(SyntaxError(format!(
            "Expected {}, instead found `{}`.",
            Keyword::Create,
            wrong_token
        ))),
        None => Err(SyntaxError(format!(
            "Expected {}, instead found end of statement.",
            Keyword::Create
        ))),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CreateTableStatement {
    table: TableDefinition,
    if_not_exists: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InsertStatement {
    table_name: String,
    column_names: Vec<String>,
    values: Vec<DataInstance>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    CreateTable(CreateTableStatement),
    Insert(InsertStatement),
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parsing_works_with_create_table() {
        let statement = "CREATE TABLE IF NOT EXISTS test (
            server_id nullable(UINT64),
            hash UINT128,
            sent_at TIMESTAMP
        );";

        let detected_statement = parse_statement(&statement).unwrap();

        assert_eq!(
            detected_statement,
            Statement::CreateTable(CreateTableStatement {
                table: TableDefinition {
                    name: "test".to_string(),
                    columns: vec![
                        ColumnDefinition {
                            name: "server_id".to_string(),
                            data_type: DataType {
                                raw_type: DataTypeRaw::UInt64,
                                is_nullable: true
                            }
                        },
                        ColumnDefinition {
                            name: "hash".to_string(),
                            data_type: DataType {
                                raw_type: DataTypeRaw::UInt128,
                                is_nullable: false
                            }
                        },
                        ColumnDefinition {
                            name: "sent_at".to_string(),
                            data_type: DataType {
                                raw_type: DataTypeRaw::Timestamp,
                                is_nullable: false
                            }
                        },
                    ]
                },
                if_not_exists: true
            })
        )
    }
}
