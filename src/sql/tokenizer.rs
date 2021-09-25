use std::str::FromStr;

use super::errors::*;
use super::expects::*;
use std::fmt::{self, Debug};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Delimiter {
    Comma,
    Semicolon,
    SingleQuote,
    DoubleQuote,
    ParenthesisOpening,
    ParenthesisClosing,
}

impl Delimiter {
    /// Delimiting characters that are devoid of semantics and are only used to split the text.
    const TRANSPARENT_CHARS: &'static [char] = &[' ', '\t', '\n', '\r'];
    /// Delimiting characters that affect statement meaning. Each one is a Delimiter variant.
    const MEANINGFUL_CHARS: &'static [char] = &[',', ';', '\'', '"', '(', ')'];
}

impl fmt::Display for Delimiter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Comma => ",",
                Self::Semicolon => ";",
                Self::SingleQuote => "'",
                Self::DoubleQuote => "\"",
                Self::ParenthesisOpening => "(",
                Self::ParenthesisClosing => ")",
            }
        )
    }
}

impl FromStr for Delimiter {
    type Err = String;

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        match candidate {
            "," => Ok(Self::Comma),
            ";" => Ok(Self::Semicolon),
            "'" => Ok(Self::SingleQuote),
            "\"" => Ok(Self::DoubleQuote),
            "(" => Ok(Self::ParenthesisOpening),
            ")" => Ok(Self::ParenthesisClosing),
            _ => Err(format!(
                "`{}` does not refer to a meaningful delimiter",
                candidate
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConstToken {
    Create,
    Table,
    If,
    Not,
    Exists,
    Nullable,
    Primary,
    Metric,
    Key,
}

impl fmt::Display for ConstToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ConstToken::Create => "CREATE",
                ConstToken::Table => "TABLE",
                ConstToken::If => "IF",
                ConstToken::Not => "NOT",
                ConstToken::Exists => "EXISTS",
                ConstToken::Nullable => "NULLABLE",
                ConstToken::Primary => "PRIMARY",
                ConstToken::Metric => "METRIC",
                ConstToken::Key => "KEY",
            }
        )
    }
}

impl FromStr for ConstToken {
    type Err = String;

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        match candidate.to_lowercase().as_str() {
            "create" => Ok(Self::Create),
            "table" => Ok(Self::Table),
            "if" => Ok(Self::If),
            "not" => Ok(Self::Not),
            "exists" => Ok(Self::Exists),
            "nullable" => Ok(Self::Nullable),
            "primary" => Ok(Self::Primary),
            "metric" => Ok(Self::Metric),
            "key" => Ok(Self::Key),
            _ => Err(format!("`{}` does not refer to a const token", candidate)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ValueType {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Timestamp,
    VarChar,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValueInstance {
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    Timestamp(u64),
    VarChar(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ValueTypeWrapped {
    pub value_type: ValueType,
    pub is_nullable: bool,
}

impl FromStr for ValueType {
    type Err = String;

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        match candidate.to_lowercase().as_str() {
            "uint8" => Ok(Self::UInt8),
            "uint16" => Ok(Self::UInt16),
            "uint32" => Ok(Self::UInt32),
            "uint64" => Ok(Self::UInt64),
            "uint128" => Ok(Self::UInt128),
            "timestamp" => Ok(Self::Timestamp),
            "varchar" => Ok(Self::VarChar),
            _ => Err(format!(
                "`{}` does not refer to a supported type",
                candidate
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Delimiting(Delimiter),
    Const(ConstToken),
    Type(ValueType),
    Arbitrary(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Delimiting(value) => fmt::Display::fmt(&value, f),
            Token::Const(value) => fmt::Display::fmt(&value, f),
            Token::Type(value) => value.fmt(f),
            Token::Arbitrary(value) => fmt::Display::fmt(&value, f),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TokenSequence<'a>(pub &'a [Token]);

impl<'a> fmt::Display for TokenSequence<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<_> = self.0.into_iter().map(|i| i.to_string()).collect();
        write!(f, "{}", parts.join(" "))
    }
}

impl<'a, Idx> std::ops::Index<Idx> for TokenSequence<'a>
where
    Idx: std::slice::SliceIndex<[Token]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.0[index]
    }
}

impl FromStr for Token {
    type Err = ();

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        Ok(
            if let Ok(delimiting_token) = Delimiter::from_str(candidate) {
                Self::Delimiting(delimiting_token)
            } else if let Ok(const_token) = ConstToken::from_str(candidate) {
                Self::Const(const_token)
            } else if let Ok(suppoted_type) = ValueType::from_str(candidate) {
                Self::Type(suppoted_type)
            } else {
                Self::Arbitrary(candidate.to_string())
            },
        )
    }
}

pub fn tokenize_statement(input: &str) -> Vec<Token> {
    let raw_tokens = input
        .split(Delimiter::TRANSPARENT_CHARS)
        .filter(|element| !element.is_empty());
    let mut interpreted_tokens = Vec::<String>::new();
    for token in raw_tokens {
        let mut current_element: String = "".to_string();
        for character in token.chars() {
            if Delimiter::MEANINGFUL_CHARS.contains(&character) {
                if !current_element.is_empty() {
                    interpreted_tokens.push(current_element.clone());
                }
                interpreted_tokens.push(character.to_string());
                current_element.clear();
            } else {
                current_element.push(character);
            }
        }
        if !current_element.is_empty() {
            interpreted_tokens.push(current_element);
        }
    }
    let tokens: Vec<Token> = interpreted_tokens
        .iter()
        .map(|candidate| Token::from_str(&candidate).unwrap())
        .collect();
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenization_works_with_create_table() {
        let statement = "CREATE TABLE IF NOT EXISTS test (
            server_id nullable(UINT64),
            hash UINT128 METRIC KEY,
            sent_at TIMESTAMP
        );";

        let detected_tokens = tokenize_statement(&statement);

        let expected_tokens = [
            Token::Const(ConstToken::Create),
            Token::Const(ConstToken::Table),
            Token::Const(ConstToken::If),
            Token::Const(ConstToken::Not),
            Token::Const(ConstToken::Exists),
            Token::Arbitrary("test".to_string()),
            Token::Delimiting(Delimiter::ParenthesisOpening),
            // New line
            Token::Arbitrary("server_id".to_string()),
            Token::Const(ConstToken::Nullable),
            Token::Delimiting(Delimiter::ParenthesisOpening),
            Token::Type(ValueType::UInt64),
            Token::Delimiting(Delimiter::ParenthesisClosing),
            Token::Delimiting(Delimiter::Comma),
            // New line
            Token::Arbitrary("hash".to_string()),
            Token::Type(ValueType::UInt128),
            Token::Const(ConstToken::Metric),
            Token::Const(ConstToken::Key),
            Token::Delimiting(Delimiter::Comma),
            // New line
            Token::Arbitrary("sent_at".to_string()),
            Token::Type(ValueType::Timestamp),
            // New line
            Token::Delimiting(Delimiter::ParenthesisClosing),
            Token::Delimiting(Delimiter::Semicolon),
        ];
        assert_eq!(&detected_tokens, &expected_tokens)
    }

    #[test]
    fn tokenization_is_case_sensitive_and_insensitive_properly() {
        let statement = "CREATE table If nOT exists TEST (
            serverId nullable(Uint64)
        )";

        let detected_tokens = tokenize_statement(&statement);

        let expected_tokens = [
            Token::Const(ConstToken::Create),
            Token::Const(ConstToken::Table),
            Token::Const(ConstToken::If),
            Token::Const(ConstToken::Not),
            Token::Const(ConstToken::Exists),
            Token::Arbitrary("TEST".to_string()),
            Token::Delimiting(Delimiter::ParenthesisOpening),
            Token::Arbitrary("serverId".to_string()),
            Token::Const(ConstToken::Nullable),
            Token::Delimiting(Delimiter::ParenthesisOpening),
            Token::Type(ValueType::UInt64),
            Token::Delimiting(Delimiter::ParenthesisClosing),
            Token::Delimiting(Delimiter::ParenthesisClosing),
        ];
        assert_eq!(&detected_tokens, &expected_tokens)
    }
}
