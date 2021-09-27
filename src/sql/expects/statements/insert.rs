use crate::sql::errors::*;
use crate::sql::expects::{generic::*, semantic::*, ExpectOk, ExpectResult};
use crate::sql::tokenizer::*;

#[derive(Debug, PartialEq, Eq)]
pub struct InsertStatement {
    pub table_name: String,
    pub column_names: Vec<String>,
    pub values: Vec<Vec<Token>>,
}

/// Conjure an InsertStatement from tokens following INSERT.
pub fn expect_insert<'t>(tokens: &'t [Token]) -> ExpectResult<'t, InsertStatement> {
    let ExpectOk { rest, .. } = expect_token_value(tokens, &TokenValue::Const(Keyword::Into))?;
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_table_name,
        outcome: table_name,
    } = expect_identifier(rest)?;
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_column_names,
        outcome: column_names,
    } = expect_enclosed_comma_separated(rest, expect_identifier)?;
    let ExpectOk { rest, .. } = expect_token_value(rest, &TokenValue::Const(Keyword::Values))?;
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_values,
        outcome: values,
    } = expect_enclosed_comma_separated(rest, expect_identity)?;
    Ok(ExpectOk {
        rest,
        tokens_consumed_count: 2 // +2 to account for INTO + VALUES
            + tokens_consumed_count_table_name
            + tokens_consumed_count_column_names + tokens_consumed_count_values,
        outcome: InsertStatement {
            table_name,
            column_names,
            values,
        },
    })
}
