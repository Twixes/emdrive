use crate::constructs::components::{
    DataDefinition, DataInstance, DataInstanceRaw, DataType, DataTypeRaw, Expression,
};
use crate::constructs::functions::Function;
use crate::sql::errors::*;
use crate::sql::expects::{generic::*, ExpectOk, ExpectResult};
use crate::sql::tokenizer::*;

pub fn expect_identifier(tokens: &[Token]) -> ExpectResult<String> {
    let ExpectOk {
        outcome: found_token,
        ..
    } = expect_next_token(tokens, &"an identifier")?;
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

pub fn expect_data_type_raw(tokens: &[Token]) -> ExpectResult<DataTypeRaw> {
    let ExpectOk {
        outcome: found_token,
        ..
    } = expect_next_token(tokens, &"a data type")?;
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
            "Expected a data type, instead found {}.",
            wrong_token
        ))),
    }
}

pub fn expect_data_type(tokens: &[Token]) -> ExpectResult<DataType> {
    let is_nullable = matches!(
        expect_token_value(tokens, &TokenValue::Const(Keyword::Nullable)),
        Ok(_)
    );
    let ExpectOk {
        rest,
        tokens_consumed_count,
        outcome: data_type,
    } = if is_nullable {
        expect_enclosed(
            &tokens[1..],
            expect_data_type_raw,
            Delimiter::ParenthesisOpening,
            Delimiter::ParenthesisClosing,
        )?
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

pub fn expect_data_instance(tokens: &[Token]) -> ExpectResult<DataInstance> {
    let ExpectOk {
        rest,
        tokens_consumed_count,
        outcome: found_token,
    } = expect_next_token(tokens, &"a value")?;
    match found_token {
        Token {
            value: TokenValue::Const(Keyword::Null),
            ..
        } => Ok(ExpectOk {
            rest,
            tokens_consumed_count,
            outcome: DataInstance::Null,
        }),
        Token {
            value: TokenValue::String(found_string),
            ..
        } => Ok(ExpectOk {
            rest,
            tokens_consumed_count,
            outcome: DataInstance::Direct(DataInstanceRaw::String(found_string.into())),
        }),
        Token {
            value: TokenValue::Arbitrary(found_number_candidate),
            ..
        } => match found_number_candidate.parse::<u32>() {
            // UInt32 is the default integer type
            Ok(found_number) => Ok(ExpectOk {
                rest,
                tokens_consumed_count,
                outcome: DataInstance::Direct(DataInstanceRaw::UInt32(found_number)),
            }),
            Err(_) => Err(SyntaxError(format!(
                "Expected a value, instead found {}.",
                found_number_candidate
            ))),
        },
        wrong_token => Err(SyntaxError(format!(
            "Expected a value, instead found {}.",
            wrong_token
        ))),
    }
}

pub fn expect_function_call(tokens: &[Token]) -> ExpectResult<Function> {
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_call,
        outcome: found_token,
    } = expect_next_token(tokens, &"a function name")?;
    match found_token {
        Token {
            value: TokenValue::Function(found_function),
            ..
        } => {
            let ExpectOk {
                rest,
                tokens_consumed_count: tokens_consumed_count_parentheses,
                outcome: _,
            } = expect_token_values_sequence(
                rest,
                &[
                    TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                    TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                ],
            )?;
            Ok(ExpectOk {
                rest,
                tokens_consumed_count: tokens_consumed_count_call
                    + tokens_consumed_count_parentheses,
                outcome: found_function.to_owned(),
            })
        }
        wrong_token => Err(SyntaxError(format!(
            "Expected a function name, instead found {}.",
            wrong_token
        ))),
    }
}

pub fn expect_data_definition(tokens: &[Token]) -> ExpectResult<DataDefinition> {
    if let Ok(ExpectOk {
        rest,
        tokens_consumed_count,
        outcome: found_function,
    }) = expect_function_call(tokens)
    {
        return Ok(ExpectOk {
            rest,
            tokens_consumed_count,
            outcome: DataDefinition::FunctionCall(found_function),
        });
    }
    if let Ok(ExpectOk {
        rest,
        tokens_consumed_count,
        outcome: found_data_instance,
    }) = expect_data_instance(tokens)
    {
        return Ok(ExpectOk {
            rest,
            tokens_consumed_count,
            outcome: DataDefinition::Const(found_data_instance),
        });
    }
    if let Ok(ExpectOk {
        rest,
        tokens_consumed_count,
        outcome: found_data_instance,
    }) = expect_data_instance(tokens)
    {
        return Ok(ExpectOk {
            rest,
            tokens_consumed_count,
            outcome: DataDefinition::Const(found_data_instance),
        });
    }
    if let Ok(ExpectOk {
        rest,
        tokens_consumed_count,
        outcome: identifier,
    }) = expect_identifier(tokens)
    {
        return Ok(ExpectOk {
            rest,
            tokens_consumed_count,
            outcome: DataDefinition::Identifier(identifier),
        });
    }
    Err(SyntaxError(format!(
        "Expected a function call, a constant value or an identifier, instead found {:?}.",
        tokens.first()
    )))
}

pub fn expect_expression(tokens: &[Token]) -> ExpectResult<Expression> {
    let ExpectOk {
        rest: rest_atom,
        tokens_consumed_count: tokens_consumed_count_lhs,
        outcome: lhs_raw,
    } = expect_data_definition(tokens)?;
    let lhs = Expression::Atom(lhs_raw);
    let ExpectOk {
        rest,
        tokens_consumed_count: tokens_consumed_count_operator_and_rhs,
        outcome: operator_and_rhs,
    } = detect(
        rest_atom,
        |tokens| expect_next_token(tokens, &"an operator"),
        expect_data_definition,
        &"the right-hand side of the expression",
    )?;
    match operator_and_rhs {
        Some((
            Token {
                value: TokenValue::Delimiting(Delimiter::Equal),
                ..
            },
            rhs_raw,
        )) => Ok(ExpectOk {
            rest,
            tokens_consumed_count: tokens_consumed_count_lhs
                + tokens_consumed_count_operator_and_rhs,
            outcome: Expression::Equal(Box::new(lhs), Box::new(Expression::Atom(rhs_raw))),
        }),
        Some((
            Token {
                value: unexpected_token,
                ..
            },
            _,
        )) => Err(SyntaxError(format!(
            "Expected {}, instead found {}.",
            Delimiter::Equal,
            unexpected_token
        ))),
        None => Ok(ExpectOk {
            rest,
            tokens_consumed_count: tokens_consumed_count_lhs,
            outcome: lhs,
        }),
    }
}

#[cfg(test)]
mod expect_identifier_tests {
    use super::*;
    use pretty_assertions::assert_eq;

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

#[cfg(test)]
mod expect_data_type_wrapped_tests {
    use super::*;
    use pretty_assertions::assert_eq;

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
                "Expected closing parenthesis `)`, instead found comma `,` at line 1.".to_string()
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
                "Expected a data type, instead found arbitrary `foo` at line 1.".to_string()
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
                "Expected a data type, instead found arbitrary `bar` at line 1.".to_string()
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

#[cfg(test)]
mod expect_data_instance_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn returns_ok_string() {
        assert_eq!(
            expect_data_instance(&[Token {
                value: TokenValue::String("foo".to_string()),
                line_number: 1
            },]),
            Ok(ExpectOk {
                rest: &[][..],
                tokens_consumed_count: 1,
                outcome: DataInstance::Direct(DataInstanceRaw::String("foo".into()))
            })
        )
    }

    #[test]
    fn returns_ok_number() {
        assert_eq!(
            expect_data_instance(&[Token {
                value: TokenValue::Arbitrary("1227".to_string()),
                line_number: 1
            }]),
            Ok(ExpectOk {
                rest: &[][..],
                tokens_consumed_count: 1,
                outcome: DataInstance::Direct(DataInstanceRaw::UInt32(1227))
            })
        )
    }
}

#[cfg(test)]
mod expect_function_call_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn returns_ok_with_no_args() {
        assert_eq!(
            expect_function_call(&[
                Token {
                    value: TokenValue::Function(Function::Ulid),
                    line_number: 1
                },
                Token {
                    value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                    line_number: 1
                },
                Token {
                    value: TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                    line_number: 1
                }
            ]),
            Ok(ExpectOk {
                rest: &[][..],
                tokens_consumed_count: 3,
                outcome: Function::Ulid
            })
        )
    }

    #[test]
    fn returns_error_if_no_opening_parenthesis() {
        assert_eq!(
            expect_function_call(&[Token {
                value: TokenValue::Function(Function::Ulid),
                line_number: 1
            }]),
            Err(SyntaxError(
                "Expected opening parenthesis `(`, instead found end of statement.".to_string()
            ))
        )
    }
}
