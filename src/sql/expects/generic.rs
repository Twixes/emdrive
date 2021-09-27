use crate::sql::errors::*;
use crate::sql::expects::{ExpectOk, ExpectResult};
use crate::sql::tokenizer::*;

pub fn consume_all<'t, O>(
    tokens: &'t [Token],
    expect_something: fn(&'t [Token]) -> ExpectResult<'t, O>,
) -> Result<O, SyntaxError> {
    let ExpectOk { rest, outcome, .. } = expect_something(tokens)?;
    expect_end_of_statement(rest)?;
    Ok(outcome)
}

pub fn expect_identity<'t>(tokens: &'t [Token]) -> ExpectResult<'t, Vec<Token>> {
    Ok(ExpectOk {
        rest: &[][..],
        tokens_consumed_count: tokens.len(),
        outcome: tokens.to_vec(),
    })
}

pub fn expect_token_value<'t>(
    tokens: &'t [Token],
    expected_token_value: &TokenValue,
) -> ExpectResult<'t, ()> {
    let ExpectOk {
        rest,
        tokens_consumed_count,
        outcome: found_token,
    } = expect_next_token(tokens, expected_token_value)?;
    if &found_token.value == expected_token_value {
        Ok(ExpectOk {
            rest,
            tokens_consumed_count,
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

pub fn expect_next_token<'t>(
    tokens: &'t [Token],
    expectation_description: &dyn std::fmt::Display,
) -> ExpectResult<'t, &'t Token> {
    match tokens.first() {
        Some(found_token) => Ok(ExpectOk {
            rest: &tokens[1..],
            tokens_consumed_count: 1,
            outcome: found_token,
        }),
        None => Err(SyntaxError(format!(
            "Expected {}, instead found end of statement.",
            expectation_description
        ))),
    }
}

pub fn expect_enclosed<'t, O>(
    tokens: &'t [Token],
    expect_inside: fn(&'t [Token]) -> ExpectResult<'t, O>,
    opening: Delimiter,
    closing: Delimiter,
) -> ExpectResult<'t, O> {
    let ExpectOk { rest, .. } = expect_token_value(tokens, &TokenValue::Delimiting(opening))?;
    let ExpectOk {
        rest,
        tokens_consumed_count,
        outcome,
    } = expect_inside(rest)?;
    let ExpectOk { rest, .. } = expect_token_value(rest, &TokenValue::Delimiting(closing))?;
    let tokens_consumed_count = tokens_consumed_count + 2; // +2 to account for parentheses
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

// Expert an enclosure, the outcome being enclosure contents excluding opening and closing delimiters.
pub fn expect_enclosure<'t>(
    tokens: &'t [Token],
    opening: Delimiter,
    closing: Delimiter,
) -> ExpectResult<'t, &'t [Token]> {
    let ExpectOk { rest, .. } = expect_token_value(
        tokens,
        &TokenValue::Delimiting(Delimiter::ParenthesisOpening),
    )?;
    let mut current_enclosure_depth: usize = 0;
    let maybe_enclosure_size: Option<usize> =
        rest.iter()
            .position(|current_token| match &current_token.value {
                TokenValue::Delimiting(current_delimiter) => {
                    if current_enclosure_depth == 0 && current_delimiter == &closing {
                        return true;
                    }
                    if current_delimiter == &opening {
                        current_enclosure_depth += 1;
                    } else if current_delimiter == &closing {
                        current_enclosure_depth -= 1;
                    }
                    false
                }
                _ => false,
            });
    match maybe_enclosure_size {
        Some(enclosure_size) => Ok(ExpectOk {
            rest: &tokens[enclosure_size + 2..],
            tokens_consumed_count: enclosure_size + 2, // +2 to account for parentheses
            outcome: &rest[..enclosure_size],
        }),
        None => Err(SyntaxError(format!(
            "Expected to find a matching {} for {}, instead found end of statement.",
            closing, tokens[0]
        ))),
    }
}

pub fn expect_enclosed_comma_separated<'t, O>(
    tokens: &'t [Token],
    expect_element: fn(&'t [Token]) -> ExpectResult<'t, O>,
) -> ExpectResult<'t, Vec<O>> {
    let ExpectOk {
        rest,
        tokens_consumed_count,
        outcome: enclosure_tokens,
    } = expect_enclosure(
        tokens,
        Delimiter::ParenthesisOpening,
        Delimiter::ParenthesisClosing,
    )?;
    let mut elements = Vec::<O>::new();
    let mut previous_separator_offset: usize = 0;
    for (current_index, current_token) in enclosure_tokens.iter().enumerate() {
        if current_token.value == TokenValue::Delimiting(Delimiter::Comma) {
            if previous_separator_offset == current_index {
                expect_element(&enclosure_tokens[..previous_separator_offset])?;
            }
            elements.push(consume_all(
                &enclosure_tokens[previous_separator_offset..current_index],
                expect_element,
            )?);
            previous_separator_offset = current_index + 1;
        }
    }
    // If the final vector is empty, it means there was a trailing separator, which is generally disallowed in SQL
    let final_element_tokens = &enclosure_tokens[previous_separator_offset..];
    if final_element_tokens.len() == 0 {
        return Err(SyntaxError(format!(
            "Found disallowed trailing {}.",
            &enclosure_tokens[previous_separator_offset - 1]
        )));
    }
    elements.push(expect_element(&enclosure_tokens[previous_separator_offset..])?.outcome);
    Ok(ExpectOk {
        rest,
        tokens_consumed_count,
        outcome: elements,
    })
}

#[cfg(test)]
mod expect_token_sequence_tests {
    use super::*;
    use pretty_assertions::assert_eq;

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
    use pretty_assertions::assert_eq;

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
mod expect_enclosure_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn returns_ok() {
        assert_eq!(
            expect_enclosure(
                &[
                    Token {
                        value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Arbitrary("foo".to_string()),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Const(Keyword::Nullable),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Type(DataTypeRaw::UInt64),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Const(Keyword::Values),
                        line_number: 1
                    },
                ],
                Delimiter::ParenthesisOpening,
                Delimiter::ParenthesisClosing
            ),
            Ok(ExpectOk {
                rest: &[Token {
                    value: TokenValue::Const(Keyword::Values),
                    line_number: 1
                }][..],
                tokens_consumed_count: 7,
                outcome: &[
                    Token {
                        value: TokenValue::Arbitrary("foo".to_string()),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Const(Keyword::Nullable),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Type(DataTypeRaw::UInt64),
                        line_number: 1
                    },
                    Token {
                        value: TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                        line_number: 1
                    },
                ][..]
            })
        )
    }
}
