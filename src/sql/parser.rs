use super::errors::*;
use super::expects::*;
use super::tokenizer::*;

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
            } = expect_next_token(rest, &format!("{} or {}", Keyword::Create, Keyword::Insert))?;
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
        // ???
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
            id STRING PRIMARY KEY,
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
                            name: "id".to_string(),
                            data_type: DataType {
                                raw_type: DataTypeRaw::String,
                                is_nullable: false
                            },
                            primary_key: true,
                        },
                        ColumnDefinition {
                            name: "server_id".to_string(),
                            data_type: DataType {
                                raw_type: DataTypeRaw::UInt64,
                                is_nullable: true
                            },
                            primary_key: false,
                        },
                        ColumnDefinition {
                            name: "hash".to_string(),
                            data_type: DataType {
                                raw_type: DataTypeRaw::UInt128,
                                is_nullable: false
                            },
                            primary_key: false,
                        },
                        ColumnDefinition {
                            name: "sent_at".to_string(),
                            data_type: DataType {
                                raw_type: DataTypeRaw::Timestamp,
                                is_nullable: false
                            },
                            primary_key: false,
                        },
                    ]
                },
                if_not_exists: true
            })
        )
    }

    #[test]
    fn parsing_works_with_insert() {
        let statement = "INSERT INTO xyz (foo, bar)
        VALUES (0b11001111, 'https://twixes.com/a.png');";

        let detected_statement = parse_statement(&statement).unwrap();

        assert_eq!(
            detected_statement,
            Statement::Insert(InsertStatement {
                table_name: "xyz".to_string(),
                column_names: vec!["foo".to_string(), "bar".to_string(),],
                values: vec![
                    vec![Token {
                        value: TokenValue::Arbitrary("0b11001111".to_string()),
                        line_number: 2
                    }],
                    vec![
                        Token {
                            value: TokenValue::Delimiting(Delimiter::SingleQuote),
                            line_number: 2
                        },
                        Token {
                            value: TokenValue::Arbitrary("https://twixes.com/a.png".to_string()),
                            line_number: 2
                        },
                        Token {
                            value: TokenValue::Delimiting(Delimiter::SingleQuote),
                            line_number: 2
                        }
                    ]
                ]
            })
        )
    }
}
