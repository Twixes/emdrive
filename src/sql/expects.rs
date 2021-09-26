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

pub fn expect_token_value<'t>(
    tokens: &'t [Token],
    expected_token_value: &TokenValue,
) -> ExpectResult<'t, ()> {
    let ExpectOk { outcome: found_token, .. } = expect_next_token(tokens, expected_token_value)?;
    if &found_token.value == expected_token_value {
        Ok(ExpectOk {
            rest: &tokens[1..],
            tokens_consumed_count: 1,
            outcome: (),
        })
    } else {
        Err(SyntaxError(format!(
            "Expected {}, instead found {}.",
            expected_token_value, found_token
        )))
    }
}

pub fn expect_token_values_sequence<'t>(
    tokens: &'t [Token],
    expected_token_values: &[TokenValue],
) -> ExpectResult<'t, ()> {
    for (token_index, expected_token_value) in expected_token_values.iter().enumerate() {
        expect_token_value(&tokens[token_index..], expected_token_value)?;
    }
    let tokens_consumed_count = expected_token_values.len();
    Ok(ExpectOk {
        rest: &tokens[tokens_consumed_count..],
        tokens_consumed_count,
        outcome: (),
    })
}

pub fn expect_identifier<'t>(tokens: &'t [Token]) -> ExpectResult<'t, String> {
    let ExpectOk { outcome: found_token, .. } = expect_next_token(tokens, &"an identifier")?;
    match found_token {
        Token {
            value: TokenValue::Arbitrary(value),
            ..
        } => Ok(ExpectOk {
            rest: &tokens[1..],
            tokens_consumed_count: 1,
            outcome: value.to_owned(),
        }),
        wrong_token => Err(SyntaxError(format!(
            "Expected an identifier, instead found {}.",
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
        Some(wrong_token) => Err(SyntaxError(format!(
            "Expected end of statement, instead found {}.",
            wrong_token
        ))),
    }
}

pub fn expect_next_token<'t>(tokens: &'t [Token], expectation_description: &dyn std::fmt::Display) -> ExpectResult<'t, &'t Token> {
    match tokens.first() {
        Some(found_token) => Ok(ExpectOk {
            rest: &tokens[1..],
            tokens_consumed_count: 1,
            outcome: found_token,
        }),
        None => Err(SyntaxError(
            format!("Expected {}, instead found end of statement.", expectation_description),
        )),
    }
}

pub fn expect_enclosed<'t, O>(
    tokens: &'t [Token],
    expect_inside: fn(&'t [Token]) -> ExpectResult<'t, O>,
) -> ExpectResult<'t, O> {
    let ExpectOk { rest, .. } = expect_token_value(
        tokens,
        &TokenValue::Delimiting(Delimiter::ParenthesisOpening),
    )?;
    let ExpectOk {
        rest,
        tokens_consumed_count,
        outcome,
    } = expect_inside(rest)?;
    let ExpectOk { rest, .. } =
        expect_token_value(rest, &TokenValue::Delimiting(Delimiter::ParenthesisClosing))?;
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
        // Parse next element
        let ExpectOk {
            tokens_consumed_count,
            outcome,
            ..
        } = expect_element(&tokens[tokens_consumed_total_count..])?;
        tokens_consumed_total_count += tokens_consumed_count;
        outcomes.push(outcome);
        // Check for the comma (trailing comma disallowed)
        match expect_token_value(
            &tokens[tokens_consumed_total_count..],
            &TokenValue::Delimiting(Delimiter::Comma),
        ) {
            Err(_) => break, // If there's no comma after this element, it's time to break out of the loop
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
    pub data_type: DataType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TableDefinition {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

pub fn expect_data_type_raw<'t>(tokens: &'t [Token]) -> ExpectResult<'t, DataTypeRaw> {
    let ExpectOk { outcome: found_token, .. } = expect_next_token(tokens, &"a data type")?;
    match found_token {
        Token {
            value: TokenValue::Type(found_data_type),
            ..
        } => Ok(ExpectOk {
            rest: &tokens[1..],
            tokens_consumed_count: 1,
            outcome: *found_data_type,
        }),
        wrong_token => Err(SyntaxError(format!(
            "Expected a type, instead found {}.",
            wrong_token
        ))),
    }
}

pub fn expect_data_type<'t>(tokens: &'t [Token]) -> ExpectResult<'t, DataType> {
    let is_nullable = match expect_token_value(tokens, &TokenValue::Const(Keyword::Nullable)) {
        Ok(..) => true,
        _ => false,
    };
    let ExpectOk {
        rest,
        tokens_consumed_count,
        outcome: data_type,
    } = if is_nullable {
        expect_enclosed(&tokens[1..], expect_data_type_raw)?
    } else {
        expect_data_type_raw(tokens)?
    };
    Ok(ExpectOk {
        rest,
        tokens_consumed_count: usize::from(is_nullable) + tokens_consumed_count,
        outcome: DataType {
            raw_type: data_type,
            is_nullable,
        },
    })
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
            expect_token_values_sequence(
                &[
                    Token {
                        value: TokenValue::Const(Keyword::If),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Const(Keyword::Not),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Const(Keyword::Exists),
                        line_number: 1
                    }
                ],
                &[
                    TokenValue::Const(Keyword::If),
                    TokenValue::Const(Keyword::Not),
                    TokenValue::Const(Keyword::Exists),
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
            expect_token_values_sequence(
                &[
                    Token {
                        value: TokenValue::Const(Keyword::If),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Const(Keyword::Not),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Arbitrary("xyz".to_string()),
                        line_number: 1
                    }
                ],
                &[
                    TokenValue::Const(Keyword::If),
                    TokenValue::Const(Keyword::Not),
                    TokenValue::Const(Keyword::Exists),
                ]
            ),
            Err(SyntaxError(
                "Expected keyword `EXISTS`, instead found arbitrary `xyz` at line 1.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_too_few_tokens() {
        assert_eq!(
            expect_token_values_sequence(
                &[Token {
                    value: TokenValue::Const(Keyword::If),
                    line_number: 1
                }],
                &[
                    TokenValue::Const(Keyword::If),
                    TokenValue::Const(Keyword::Not),
                    TokenValue::Const(Keyword::Exists),
                ]
            ),
            Err(SyntaxError(
                "Expected keyword `NOT`, instead found end of statement.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_eos() {
        assert_eq!(
            expect_token_values_sequence(
                &[],
                &[
                    TokenValue::Const(Keyword::If),
                    TokenValue::Const(Keyword::Not),
                    TokenValue::Const(Keyword::Exists),
                ]
            ),
            Err(SyntaxError(
                "Expected keyword `IF`, instead found end of statement.".to_string()
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
            expect_token_value(
                &[
                    Token {
                        value: TokenValue::Const(Keyword::Primary),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Arbitrary("foo".to_string()),
                        line_number: 1
                    }
                ],
                &TokenValue::Const(Keyword::Primary)
            ),
            Ok(ExpectOk {
                rest: &[Token {
                    value: TokenValue::Arbitrary("foo".to_string()),
                    line_number: 1
                }][..],
                tokens_consumed_count: 1,
                outcome: ()
            })
        )
    }

    #[test]
    fn returns_error_if_const_token() {
        assert_eq!(
            expect_token_value(
                &[Token {
                    value: TokenValue::Const(Keyword::Create),
                    line_number: 1
                }],
                &TokenValue::Const(Keyword::Primary)
            ),
            Err(SyntaxError(
                "Expected keyword `PRIMARY`, instead found keyword `CREATE` at line 1.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_eos() {
        assert_eq!(
            expect_token_value(&[], &TokenValue::Const(Keyword::Primary)),
            Err(SyntaxError(
                "Expected keyword `PRIMARY`, instead found end of statement.".to_string()
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
            expect_identifier(&[Token {
                value: TokenValue::Arbitrary("foo".to_string()),
                line_number: 1
            }]),
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
            expect_identifier(&[Token {
                value: TokenValue::Const(Keyword::Create),
                line_number: 1
            }]),
            Err(SyntaxError(
                "Expected an identifier, instead found keyword `CREATE` at line 1.".to_string()
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
mod expect_data_type_wrapped_tests {
    use super::*;

    #[test]
    fn returns_ok_uint64() {
        assert_eq!(
            expect_data_type(&[Token {
                value: TokenValue::Type(DataTypeRaw::UInt64),
                line_number: 1
            }]),
            Ok(ExpectOk {
                rest: &[][..],
                tokens_consumed_count: 1,
                outcome: DataType {
                    raw_type: DataTypeRaw::UInt64,
                    is_nullable: false
                }
            })
        )
    }

    #[test]
    fn returns_ok_nullable_timestamp() {
        assert_eq!(
            expect_data_type(&[
                Token {
                    value: TokenValue::Const(Keyword::Nullable),
                    line_number: 1
                },
                Token {
                    value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                    line_number: 1
                },
                Token {
                    value: TokenValue::Type(DataTypeRaw::Timestamp),
                    line_number: 1
                },
                Token {
                    value: TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                    line_number: 1
                }
            ]),
            Ok(ExpectOk {
                rest: &[][..],
                tokens_consumed_count: 4,
                outcome: DataType {
                    raw_type: DataTypeRaw::Timestamp,
                    is_nullable: true
                }
            })
        )
    }

    #[test]
    fn returns_error_if_nullable_not_closed() {
        assert_eq!(
            expect_data_type(&[
                Token {
                    value: TokenValue::Const(Keyword::Nullable),
                    line_number: 1
                },
                Token {
                    value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                    line_number: 1
                },
                Token {
                    value: TokenValue::Type(DataTypeRaw::Timestamp),
                    line_number: 1
                },
                Token {
                    value: TokenValue::Delimiting(Delimiter::Comma),
                    line_number: 1
                }
            ]),
            Err(SyntaxError(
                "Expected delimiter `)`, instead found delimiter `,` at line 1.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_no_type() {
        assert_eq!(
            expect_data_type(&[Token {
                value: TokenValue::Arbitrary("foo".to_string()),
                line_number: 1
            }]),
            Err(SyntaxError(
                "Expected a type, instead found arbitrary `foo` at line 1.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_neos() {
        assert_eq!(
            expect_data_type(&[]),
            Err(SyntaxError(
                "Expected a data type, instead found end of statement.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_no_type_but_nullable() {
        assert_eq!(
            expect_data_type(&[
                Token {
                    value: TokenValue::Const(Keyword::Nullable),
                    line_number: 1
                },
                Token {
                    value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                    line_number: 1
                },
                Token {
                    value: TokenValue::Arbitrary("bar".to_string()),
                    line_number: 1
                }
            ]),
            Err(SyntaxError(
                "Expected a type, instead found arbitrary `bar` at line 1.".to_string()
            ))
        )
    }

    #[test]
    fn returns_error_if_eos_but_nullable() {
        assert_eq!(
            expect_data_type(&[
                Token {
                    value: TokenValue::Const(Keyword::Nullable),
                    line_number: 1
                },
                Token {
                    value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                    line_number: 1
                }
            ]),
            Err(SyntaxError(
                "Expected a data type, instead found end of statement.".to_string()
            ))
        )
    }
}
