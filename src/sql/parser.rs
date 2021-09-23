use crate::sql::errors::*;
use crate::sql::tokenizer::*;

fn parse_create(tokens: &[Token]) -> Result<Statement, SyntaxError> {
    let mut if_not_exists = false;
    let first = &tokens[0];
    let rest = &tokens[1..];
    Err(SyntaxError("Not implemented!".to_string()))
}

pub fn parse_statement(input: &str) -> Result<Statement, SyntaxError> {
    let tokens = tokenize_statement(input);
    let mut current_expected_possibilities: Vec<Token> = vec![Token::Const(ConstToken::Create)];
    let first = &tokens[0];
    let rest = &tokens[1..];
    match first {
        Token::Const(ConstToken::Create) => parse_create(rest),
        _ => Err(SyntaxError(format!(
            "Expected one of: {}, got {}",
            ConstToken::Create,
            first
        ))),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ColumnDefinition {
    name: String,
    value_type: ValueTypeWrapped,
}

#[derive(Debug, PartialEq, Eq)]
struct TableDefinition {
    name: String,
    columns: Vec<ColumnDefinition>,
}

#[derive(Debug, PartialEq, Eq)]
struct CreateTableStatement {
    table: TableDefinition,
    if_not_exists: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    CreateTable(CreateTableStatement),
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn parsing_works_with_create_table() {
        let statement = "CREATE TABLE IF NOT EXISTS test (
            server_id nullable(UINT64),
            hash UINT128 METRIC KEY,
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
                            value_type: ValueTypeWrapped {
                                value_type: ValueType::UInt64,
                                is_nullable: true
                            }
                        },
                        ColumnDefinition {
                            name: "hash".to_string(),
                            value_type: ValueTypeWrapped {
                                value_type: ValueType::UInt128,
                                is_nullable: false
                            }
                        },
                        ColumnDefinition {
                            name: "sent_at".to_string(),
                            value_type: ValueTypeWrapped {
                                value_type: ValueType::Timestamp,
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
