use crate::sql::errors::*;
use crate::sql::tokenizer::*;

pub fn expect_if_not_exists(tokens: &[Token]) -> Result<(), SyntaxError> {
    let expected_tokens: &[Token] = &[
        Token::Const(ConstToken::If),
        Token::Const(ConstToken::Not),
        Token::Const(ConstToken::Exists),
    ];
    let found_token_count = tokens.len();
    if found_token_count == 0 {
        Err(SyntaxError(format!(
            "Expected \"{}\", instead found end of statement.",
            TokenSequence(expected_tokens)
        )))
    } else if found_token_count < 3 {
        Err(SyntaxError(format!(
            "Expected \"{}\", instead found just \"{}\".",
            TokenSequence(expected_tokens),
            TokenSequence(tokens)
        )))
    } else {
        let found_tokens: &[Token] = &tokens[..3];
        if found_tokens == expected_tokens {
            Ok(())
        } else {
            Err(SyntaxError(format!(
                "Expected \"{}\", instead found \"{}\".",
                TokenSequence(expected_tokens),
                TokenSequence(found_tokens)
            )))
        }
    }
}

pub fn expect_identifier(tokens: &[Token]) -> Result<String, SyntaxError> {
    match tokens.first() {
        Some(Token::Arbitrary(value)) => Ok(value.to_owned()),
        Some(wrong_token) => Err(SyntaxError(format!(
            "Expected a string identifier, instead found \"{}\".",
            wrong_token
        ))),
        None => Err(SyntaxError(
            "Expected a string identifier, instead found end of statement.".to_string(),
        )),
    }
}

pub fn expect_end_of_statement(tokens: &[Token]) -> Result<(), SyntaxError> {
    match tokens.first() {
        None => Ok(()),
        Some(Token::Delimiting(Delimiter::Semicolon)) => {
            if tokens.len() > 1 {
                Err(SyntaxError("Found tokens after a semicolon! Only a single statement at once can be provided.".to_string()))
            } else {
                Ok(())
            }
        }
        Some(wrong_token) => Err(SyntaxError(format!(
            "Expected no more tokens or a semicolon, instead found {}.",
            wrong_token
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expect_if_not_exists_returns_ok() {
        assert_eq!(
            expect_if_not_exists(&[
                Token::Const(ConstToken::If),
                Token::Const(ConstToken::Not),
                Token::Const(ConstToken::Exists)
            ]),
            Ok(())
        )
    }

    #[test]
    fn expect_if_not_exists_returns_error_if_third_token_invalid() {
        assert_eq!(
            expect_if_not_exists(&[
                Token::Const(ConstToken::If),
                Token::Const(ConstToken::Not),
                Token::Arbitrary("xyz".to_string())
            ]),
            Err(SyntaxError(
                "Expected \"IF NOT EXISTS\", instead found \"IF NOT xyz\".".to_string()
            ))
        )
    }

    #[test]
    fn expect_if_not_exists_returns_error_if_only_one_token() {
        assert_eq!(
            expect_if_not_exists(&[Token::Const(ConstToken::If)]),
            Err(SyntaxError(
                "Expected \"IF NOT EXISTS\", instead found just \"IF\".".to_string()
            ))
        )
    }

    #[test]
    fn expect_if_not_exists_returns_error_if_eos() {
        assert_eq!(
            expect_if_not_exists(&[]),
            Err(SyntaxError(
                "Expected \"IF NOT EXISTS\", instead found end of statement.".to_string()
            ))
        )
    }
}
