use super::errors::*;
use super::tokenizer::*;

#[derive(Debug, PartialEq, Eq)]
pub struct ExpectOk<'t, O> {
    pub rest: &'t [Token],
    pub tokens_consumed_count: usize,
    pub outcome: O,
}
pub type ExpectResult<'t, O> = Result<ExpectOk<'t, O>, SyntaxError>;

// Generic expects

pub fn expect_token_sequence<'t>(
    tokens: &'t [Token],
    expected_tokens: &[Token],
) -> ExpectResult<'t, ()> {
    let expected_token_count = expected_tokens.len();
    let found_token_count = tokens.len();
    if found_token_count == 0 {
        Err(SyntaxError(format!(
            "Expected `{}`, instead found end of statement.",
            TokenSequence(expected_tokens)
        )))
    } else if found_token_count < expected_token_count {
        Err(SyntaxError(format!(
            "Expected `{}`, instead found just `{}`.",
            TokenSequence(expected_tokens),
            TokenSequence(tokens)
        )))
    } else {
        let found_tokens: &[Token] = &tokens[..expected_token_count];
        if found_tokens == expected_tokens {
            Ok(ExpectOk {
                rest: &tokens[expected_token_count..],
                tokens_consumed_count: expected_token_count,
                outcome: (),
            })
        } else {
            Err(SyntaxError(format!(
                "Expected `{}`, instead found `{}`.",
                TokenSequence(expected_tokens),
                TokenSequence(found_tokens)
            )))
        }
    }
}

pub fn expect_token_single<'t>(
    tokens: &'t [Token],
    expected_token: &Token,
) -> ExpectResult<'t, ()> {
    match tokens.first() {
        None | Some(Token::Delimiting(Delimiter::Semicolon)) => Err(SyntaxError(format!(
            "Expected `{}`, instead found end of statement.",
            expected_token
        ))),
        Some(found_token) => {
            if found_token == expected_token {
                Ok(ExpectOk {
                    rest: &tokens[1..],
                    tokens_consumed_count: 1,
                    outcome: (),
                })
            } else {
                Err(SyntaxError(format!(
                    "Expected `{}`, instead found `{}`.",
                    expected_token, found_token
                )))
            }
        }
    }
}

pub fn expect_identifier<'t>(tokens: &'t [Token]) -> ExpectResult<'t, String> {
    match tokens.first() {
        None | Some(Token::Delimiting(Delimiter::Semicolon)) => Err(SyntaxError(
            "Expected an identifier, instead found end of statement.".to_string(),
        )),
        Some(Token::Arbitrary(value)) => Ok(ExpectOk {
            rest: &tokens[1..],
            tokens_consumed_count: 1,
            outcome: value.to_owned(),
        }),
        Some(wrong_token) => Err(SyntaxError(format!(
            "Expected an identifier, instead found `{}`.",
            wrong_token
        ))),
    }
}

pub fn expect_end_of_statement<'t>(tokens: &'t [Token]) -> ExpectResult<'t, ()> {
    match tokens.first() {
        None => Ok(ExpectOk {
            rest: tokens,
            tokens_consumed_count: 0,
            outcome: (),
        }),
        Some(Token::Delimiting(Delimiter::Semicolon)) => {
            if tokens.len() > 1 {
                Err(SyntaxError("Found tokens after a semicolon! Only a single statement at once can be provided.".to_string()))
            } else {
                Ok(ExpectOk {
                    rest: &tokens[1..],
                    tokens_consumed_count: 1,
                    outcome: (),
                })
            }
        }
        Some(wrong_token) => Err(SyntaxError(format!(
            "Expected no more tokens or a semicolon, instead found {}.",
            wrong_token
        ))),
    }
}

pub fn expect_enclosed<'t, O>(
    tokens: &'t [Token],
    expect_inside: fn(&'t [Token]) -> ExpectResult<'t, O>,
) -> ExpectResult<'t, O> {
    let ExpectOk { rest, .. } =
        expect_token_single(tokens, &Token::Delimiting(Delimiter::ParenthesisOpening))?;
    let ExpectOk {
        rest,
        tokens_consumed_count,
        outcome,
    } = expect_inside(rest)?;
    let ExpectOk { rest, .. } =
        expect_token_single(rest, &Token::Delimiting(Delimiter::ParenthesisClosing))?;
    let tokens_consumed_count = tokens_consumed_count + 2; // Account for parentheses
    Ok(ExpectOk {
        rest,
        tokens_consumed_count,
        outcome,
    })
}

pub fn expect_comma_separated<'t, O>(
    tokens: &'t [Token],
    expect_element: fn(&'t [Token]) -> ExpectResult<'t, O>,
) -> ExpectResult<'t, Vec<O>> {
    let mut tokens_consumed_total_count = 0;
    let mut outcomes = Vec::<O>::new();
    loop {
        match expect_element(&tokens[tokens_consumed_total_count..]) {
            Err(_) => break,
            Ok(ExpectOk {
                tokens_consumed_count,
                outcome,
                ..
            }) => {
                tokens_consumed_total_count += tokens_consumed_count;
                outcomes.push(outcome);
            }
        }
        match expect_token_single(
            &tokens[tokens_consumed_total_count..],
            &Token::Delimiting(Delimiter::Comma),
        ) {
            Err(_) => break,
            _ => {
                tokens_consumed_total_count += 1;
            }
        }
    }
    Ok(ExpectOk {
        rest: &tokens[tokens_consumed_total_count..],
        tokens_consumed_count: tokens_consumed_total_count,
        outcome: outcomes,
    })
}

// Semantic expects

#[derive(Debug, PartialEq, Eq)]
pub struct ColumnDefinition {
    pub name: String,
    pub value_type: ValueTypeWrapped,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TableDefinition {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

pub fn expect_value_type_wrapped<'t>(tokens: &'t [Token]) -> ExpectResult<'t, ValueTypeWrapped> {
    let mut tokens_consumed_count = 0;
    let mut is_nullable = false;
    let value_type: ValueType;
    let nullable_sequence = &[
        Token::Const(ConstToken::Nullable),
        Token::Delimiting(Delimiter::ParenthesisOpening),
    ];
    match expect_token_sequence(tokens, nullable_sequence) {
        Ok(ExpectOk { outcome: (), .. }) => {
            tokens_consumed_count += nullable_sequence.len();
            is_nullable = true;
        }
        _ => (),
    };
    match tokens[tokens_consumed_count..].first() {
        None | Some(Token::Delimiting(Delimiter::Semicolon)) => {
            return Err(SyntaxError(if is_nullable {
                "Expected a type, instead found end of statement.".to_string()
            } else {
                "Expected a type or `NULLABLE(`, instead found end of statement.".to_string()
            }))
        }
        Some(Token::Type(found_value_type)) => {
            tokens_consumed_count += 1;
            value_type = *found_value_type;
        }
        Some(wrong_token) => {
            return Err(SyntaxError(if is_nullable {
                format!("Expected a type, instead found `{}`.", wrong_token)
            } else {
                format!(
                    "Expected a type or `NULLABLE(`, instead found `{}`.",
                    wrong_token
                )
            }))
        }
    };
    if is_nullable {
        match tokens[tokens_consumed_count..].first() {
            None | Some(Token::Delimiting(Delimiter::Semicolon)) => {
                return Err(SyntaxError(
                    "Expected a closing parenthesis, instead found end of statement.".to_string(),
                ))
            }
            Some(Token::Delimiting(Delimiter::ParenthesisClosing)) => Ok(ExpectOk {
                rest: &tokens[tokens_consumed_count + 1..],
                tokens_consumed_count: tokens_consumed_count + 1,
                outcome: ValueTypeWrapped {
                    value_type,
                    is_nullable,
                },
            }),
            Some(wrong_token) => {
                return Err(SyntaxError(format!(
                    "Expected a closing parenthesis, instead found `{}`.",
                    wrong_token
                )))
            }
        }
    } else {
        Ok(ExpectOk {
            rest: &tokens[tokens_consumed_count..],
            tokens_consumed_count,
            outcome: ValueTypeWrapped {
                value_type,
                is_nullable,
            },
        })
    }
}

pub fn expect_column_definition<'t>(tokens: &'t [Token]) -> ExpectResult<'t, ColumnDefinition> {
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_name,
        outcome: name,
    } = expect_identifier(tokens)?;
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_value_type,
        outcome: value_type,
    } = expect_value_type_wrapped(rest)?;
    Ok(ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_name + tokens_consumed_count_value_type,
        outcome: ColumnDefinition { name, value_type },
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
    } = expect_enclosed(rest, |tokens_enclosed| {
        Ok(expect_comma_separated(
            tokens_enclosed,
            expect_column_definition,
        )?)
    })?;
    Ok(ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_name + tokens_consumed_count_columns,
        outcome: TableDefinition { name, columns },
    })
}

// Generic expect tests

#[cfg(test)]
mod expect_token_sequence_tests {
    use super::*;

    #[test]
    fn returns_ok() {
        assert_eq!(
            expect_token_sequence(
                &[
                    Token::Const(ConstToken::If),
                    Token::Const(ConstToken::Not),
                    Token::Const(ConstToken::Exists)
                ],
                &[
                    Token::Const(ConstToken::If),
                    Token::Const(ConstToken::Not),
                    Token::Const(ConstToken::Exists),
                ]
            ),
            Ok(ExpectOk {
                rest: &[][..],
                tokens_consumed_count: 3,
                outcome: ()
            })
        )
    }

    #[test]
    fn returns_error_if_third_token_invalid() {
        assert_eq!(
            expect_token_sequence(
                &[
                    Token::Const(ConstToken::If),
                    Token::Const(ConstToken::Not),
                    Token::Arbitrary("xyz".to_string())
                ],
                &[
                    Token::Const(ConstToken::If),
                    Token::Const(ConstToken::Not),
                    Token::Const(ConstToken::Exists),
                ]
            ),
            Err(SyntaxError(
                "Expected `IF NOT EXISTS`, instead found `IF NOT xyz`.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_too_few_tokens() {
        assert_eq!(
            expect_token_sequence(
                &[Token::Const(ConstToken::If)],
                &[
                    Token::Const(ConstToken::If),
                    Token::Const(ConstToken::Not),
                    Token::Const(ConstToken::Exists),
                ]
            ),
            Err(SyntaxError(
                "Expected `IF NOT EXISTS`, instead found just `IF`.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_eos() {
        assert_eq!(
            expect_token_sequence(
                &[],
                &[
                    Token::Const(ConstToken::If),
                    Token::Const(ConstToken::Not),
                    Token::Const(ConstToken::Exists),
                ]
            ),
            Err(SyntaxError(
                "Expected `IF NOT EXISTS`, instead found end of statement.".to_string()
            ))
        )
    }
}

#[cfg(test)]
mod expect_token_single_tests {
    use super::*;

    #[test]
    fn returns_ok() {
        assert_eq!(
            expect_token_single(
                &[
                    Token::Const(ConstToken::Primary),
                    Token::Arbitrary("foo".to_string())
                ],
                &Token::Const(ConstToken::Primary)
            ),
            Ok(ExpectOk {
                rest: &[Token::Arbitrary("foo".to_string())][..],
                tokens_consumed_count: 1,
                outcome: ()
            })
        )
    }

    #[test]
    fn returns_error_if_const_token() {
        assert_eq!(
            expect_token_single(
                &[Token::Const(ConstToken::Create)],
                &Token::Const(ConstToken::Primary)
            ),
            Err(SyntaxError(
                "Expected `PRIMARY`, instead found `CREATE`.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_eos() {
        assert_eq!(
            expect_token_single(&[], &Token::Const(ConstToken::Primary)),
            Err(SyntaxError(
                "Expected `PRIMARY`, instead found end of statement.".to_string()
            ))
        )
    }
}

#[cfg(test)]
mod expect_identifier_tests {
    use super::*;

    #[test]
    fn returns_ok() {
        assert_eq!(
            expect_identifier(&[Token::Arbitrary("foo".to_string())]),
            Ok(ExpectOk {
                rest: &[][..],
                tokens_consumed_count: 1,
                outcome: "foo".to_string()
            })
        )
    }

    #[test]
    fn returns_error_if_const_token() {
        assert_eq!(
            expect_identifier(&[Token::Const(ConstToken::Create)]),
            Err(SyntaxError(
                "Expected an identifier, instead found `CREATE`.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_eos() {
        assert_eq!(
            expect_identifier(&[]),
            Err(SyntaxError(
                "Expected an identifier, instead found end of statement.".to_string()
            ))
        )
    }
}

// Semantic expect tests

#[cfg(test)]
mod expect_value_type_wrapped_tests {
    use super::*;

    #[test]
    fn returns_ok_uint64() {
        assert_eq!(
            expect_value_type_wrapped(&[Token::Type(ValueType::UInt64)]),
            Ok(ExpectOk {
                rest: &[][..],
                tokens_consumed_count: 1,
                outcome: ValueTypeWrapped {
                    value_type: ValueType::UInt64,
                    is_nullable: false
                }
            })
        )
    }

    #[test]
    fn returns_ok_nullable_timestamp() {
        assert_eq!(
            expect_value_type_wrapped(&[
                Token::Const(ConstToken::Nullable),
                Token::Delimiting(Delimiter::ParenthesisOpening),
                Token::Type(ValueType::Timestamp),
                Token::Delimiting(Delimiter::ParenthesisClosing)
            ]),
            Ok(ExpectOk {
                rest: &[][..],
                tokens_consumed_count: 4,
                outcome: ValueTypeWrapped {
                    value_type: ValueType::Timestamp,
                    is_nullable: true
                }
            })
        )
    }

    #[test]
    fn returns_error_if_nullable_not_closed() {
        assert_eq!(
            expect_value_type_wrapped(&[
                Token::Const(ConstToken::Nullable),
                Token::Delimiting(Delimiter::ParenthesisOpening),
                Token::Type(ValueType::Timestamp),
                Token::Delimiting(Delimiter::Comma)
            ]),
            Err(SyntaxError(
                "Expected a closing parenthesis, instead found `,`.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_no_type() {
        assert_eq!(
            expect_value_type_wrapped(&[Token::Arbitrary("foo".to_string())]),
            Err(SyntaxError(
                "Expected a type or `NULLABLE(`, instead found `foo`.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_neos() {
        assert_eq!(
            expect_value_type_wrapped(&[]),
            Err(SyntaxError(
                "Expected a type or `NULLABLE(`, instead found end of statement.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_no_type_but_nullable() {
        assert_eq!(
            expect_value_type_wrapped(&[
                Token::Const(ConstToken::Nullable),
                Token::Delimiting(Delimiter::ParenthesisOpening),
                Token::Arbitrary("bar".to_string())
            ]),
            Err(SyntaxError(
                "Expected a type, instead found `bar`.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_eos_but_nullable() {
        assert_eq!(
            expect_value_type_wrapped(&[
                Token::Const(ConstToken::Nullable),
                Token::Delimiting(Delimiter::ParenthesisOpening)
            ]),
            Err(SyntaxError(
                "Expected a type, instead found end of statement.".to_string()
            ))
        )
    }
}
