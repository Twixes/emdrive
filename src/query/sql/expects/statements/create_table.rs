use crate::construct::{
    components::{ColumnDefinition, TableDefinition},
    statements::CreateTableStatement,
};
use crate::query::sql::expects::{generic::*, semantic::*, ExpectOk, ExpectResult};
use crate::query::sql::tokenizer::*;

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
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_primary_key,
        outcome: primary_key_option,
    } = optionalize(rest, |tokens| {
        expect_token_values_sequence(
            tokens,
            &[
                TokenValue::Const(Keyword::Primary),
                TokenValue::Const(Keyword::Key),
            ],
        )
    });
    Ok(ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_name
            + tokens_consumed_count_data_type
            + tokens_consumed_count_primary_key,
        outcome: ColumnDefinition {
            name,
            data_type,
            primary_key: primary_key_option.is_some(),
        },
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
    } = expect_enclosed_comma_separated(rest, expect_column_definition)?;
    Ok(ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_name + tokens_consumed_count_columns,
        outcome: TableDefinition::new(name, columns),
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
