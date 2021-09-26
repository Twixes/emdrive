use super::errors::*;
use super::expects::*;
use super::tokenizer::*;

pub fn consume_all<'t, O>(
    tokens: &'t [Token],
    expect_something: fn(&'t [Token]) -> ExpectResult<'t, O>,
) -> Result<O, SyntaxError> {
    let ExpectOk { rest, outcome, .. } = expect_something(tokens)?;
    expect_end_of_statement(rest)?;
    Ok(outcome)
}

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
        Token {
            value: TokenValue::Const(Keyword::Create),
            ..
        } => {
            let ExpectOk {
                rest,
                outcome: found_token_second,
                ..
            } = expect_next_token(rest, &format!("{} or {}", Keyword::Create, Keyword::Insert))?;
            match found_token_second {
                Token {
                    value: TokenValue::Const(Keyword::Table),
                    ..
                } => Ok(Statement::CreateTable(consume_all(
                    rest,
                    expect_create_table,
                )?)),
                wrong_token => Err(SyntaxError(format!(
                    "Expected {}, instead found {}.",
                    Keyword::Table,
                    wrong_token
                ))),
            }
        }
        Token {
            value: TokenValue::Const(Keyword::Insert),
            ..
        } => Ok(Statement::Insert(consume_all(rest, expect_insert)?)),
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
