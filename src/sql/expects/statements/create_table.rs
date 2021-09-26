use crate::sql::errors::*;
use crate::sql::expects::{generic::*, semantic::*, ExpectOk, ExpectResult};
use crate::sql::tokenizer::*;

#[derive(Debug, PartialEq, Eq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TableDefinition {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CreateTableStatement {
    pub table: TableDefinition,
    pub if_not_exists: bool,
}

pub fn expect_column_definition<'t>(tokens: &'t [Token]) -> ExpectResult<'t, ColumnDefinition> {
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_name,
        outcome: name,
    } = expect_identifier(tokens)?;
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_data_type,
        outcome: data_type,
    } = expect_data_type(rest)?;
    Ok(ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_name + tokens_consumed_count_data_type,
        outcome: ColumnDefinition { name, data_type },
    })
}

pub fn expect_table_definition<'t>(tokens: &'t [Token]) -> ExpectResult<'t, TableDefinition> {
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_name,
        outcome: name,
    } = expect_identifier(tokens)?;
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_columns,
        outcome: columns,
    } = expect_enclosed(
        rest,
        |tokens_enclosed| {
            Ok(expect_comma_separated(
                tokens_enclosed,
                expect_column_definition,
            )?)
        },
        Delimiter::ParenthesisOpening,
        Delimiter::ParenthesisClosing,
    )?;
    Ok(ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_name + tokens_consumed_count_columns,
        outcome: TableDefinition { name, columns },
    })
}

/// Conjure a CreateTableStatement from tokens following CREATE TABLE.
pub fn expect_create_table<'t>(tokens: &'t [Token]) -> ExpectResult<'t, CreateTableStatement> {
    let (if_not_exists, rest, tokens_consumed_count_if_not_exists) =
        match expect_token_values_sequence(
            tokens,
            &[
                TokenValue::Const(Keyword::If),
                TokenValue::Const(Keyword::Not),
                TokenValue::Const(Keyword::Exists),
            ],
        ) {
            Ok(ExpectOk {
                rest,
                tokens_consumed_count,
                ..
            }) => (true, rest, tokens_consumed_count),
            Err(_) => (false, tokens, 0),
        };
    let ExpectOk {
        outcome: table,
        rest,
        tokens_consumed_count: tokens_consumed_count_table_definition,
    } = expect_table_definition(rest)?;
    Ok(ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_table_definition
            + tokens_consumed_count_if_not_exists,
        outcome: CreateTableStatement {
            table,
            if_not_exists,
        },
    })
}
