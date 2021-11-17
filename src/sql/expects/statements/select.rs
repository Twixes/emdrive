use crate::constructs::statements::{SelectColumn, SelectStatement};
use crate::sql::expects::{generic::*, semantic::*, ExpectOk, ExpectResult};
use crate::sql::{tokenizer::*, SyntaxError};

pub fn expect_select_column<'t>(tokens: &'t [Token]) -> ExpectResult<'t, SelectColumn> {
    let ExpectOk {
        outcome: found_token,
        ..
    } = expect_next_token(tokens, &"a SELECT column")?;
    match found_token {
        Token {
            value: TokenValue::Arbitrary(value),
            ..
        } => Ok(ExpectOk {
            rest: &tokens[1..],
            tokens_consumed_count: 1,
            outcome: SelectColumn::Identifier(value.to_owned()),
        }),
        Token {
            value: TokenValue::Const(Keyword::Asterisk),
            ..
        } => Ok(ExpectOk {
            rest: &tokens[1..],
            tokens_consumed_count: 1,
            outcome: SelectColumn::All,
        }),
        wrong_token => Err(SyntaxError(format!(
            "Expected a SELECT column, instead found {}.",
            wrong_token
        ))),
    }
}

/// Conjure an SelectStatement from tokens following SELECT.
pub fn expect_select<'t>(tokens: &'t [Token]) -> ExpectResult<'t, SelectStatement> {
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_columns,
        outcome: columns,
    } = expect_comma_separated(tokens, expect_select_column)?;
    let ExpectOk { rest, .. } = expect_token_value(rest, &TokenValue::Const(Keyword::From))?;
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_table_name,
        outcome: table_name,
    } = expect_identifier(rest)?;
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_where_clause,
        outcome: maybe_where_clause,
    } = detect(
        rest,
        |tokens| expect_token_value(tokens, &TokenValue::Const(Keyword::Where)),
        expect_expression,
    )?;
    Ok(ExpectOk {
        rest,
        tokens_consumed_count: 1 // +1 to account for FROM
            + tokens_consumed_columns
            + tokens_consumed_count_table_name + tokens_consumed_count_where_clause,
        outcome: SelectStatement {
            columns,
            source: table_name,
            where_clause: maybe_where_clause.and_then(|(_, where_clause)| Some(where_clause)),
        },
    })
}
