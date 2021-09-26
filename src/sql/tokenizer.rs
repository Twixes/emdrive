use std::str::FromStr;

use super::errors::*;
use std::fmt::{self, Debug};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Delimiter {
    Comma,
    SingleQuote,
    DoubleQuote,
    ParenthesisOpening,
    ParenthesisClosing,
}

impl Delimiter {
    /// Delimiting characters that affect statement meaning. Each one is a Delimiter variant.
    const MEANINGFUL_CHARS: &'static [char] = &[',', '\'', '"', '(', ')'];
    const STATEMENT_SEPARATOR: char = ';';
}

impl fmt::Display for Delimiter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Comma => "comma `,`",
                Self::SingleQuote => "single quote `'`",
                Self::DoubleQuote => "double quote `\"`",
                Self::ParenthesisOpening => "opening parenthesis `(`",
                Self::ParenthesisClosing => "closing parenthesis `)`",
            }
        )
    }
}

impl FromStr for Delimiter {
    type Err = String;

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        match candidate {
            "," => Ok(Self::Comma),
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
pub enum Keyword {
    Create,
    Insert,
    Into,
    Values,
    Table,
    If,
    Not,
    Exists,
    Nullable,
    Primary,
    Metric,
    Key,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "keyword `{}`",
            match self {
                Keyword::Create => "CREATE",
                Keyword::Insert => "INSERT",
                Keyword::Into => "INTO",
                Keyword::Values => "VALUES",
                Keyword::Table => "TABLE",
                Keyword::If => "IF",
                Keyword::Not => "NOT",
                Keyword::Exists => "EXISTS",
                Keyword::Nullable => "NULLABLE",
                Keyword::Primary => "PRIMARY",
                Keyword::Metric => "METRIC",
                Keyword::Key => "KEY",
            }
        )
    }
}

impl FromStr for Keyword {
    type Err = String;

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        match candidate.to_lowercase().as_str() {
            "create" => Ok(Self::Create),
            "insert" => Ok(Self::Insert),
            "into" => Ok(Self::Into),
            "values" => Ok(Self::Values),
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
pub enum DataTypeRaw {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Timestamp,
    String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DataInstance {
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    Timestamp(u64),
    String(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataType {
    pub raw_type: DataTypeRaw,
    pub is_nullable: bool,
}

impl FromStr for DataTypeRaw {
    type Err = String;

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        match candidate.to_lowercase().as_str() {
            "uint8" => Ok(Self::UInt8),
            "uint16" => Ok(Self::UInt16),
            "uint32" => Ok(Self::UInt32),
            "uint64" => Ok(Self::UInt64),
            "uint128" => Ok(Self::UInt128),
            "timestamp" => Ok(Self::Timestamp),
            "string" => Ok(Self::String),
            _ => Err(format!(
                "`{}` does not refer to a supported type",
                candidate
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenValue {
    Delimiting(Delimiter),
    Const(Keyword),
    Type(DataTypeRaw),
    Arbitrary(String),
}

impl fmt::Display for TokenValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Delimiting(value) => fmt::Display::fmt(&value, f),
            Self::Const(value) => fmt::Display::fmt(&value, f),
            Self::Type(value) => value.fmt(f),
            Self::Arbitrary(value) => write!(f, "arbitrary `{}`", value),
        }
    }
}

impl FromStr for TokenValue {
    type Err = ();

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        Ok(
            if let Ok(delimiting_token) = Delimiter::from_str(candidate) {
                Self::Delimiting(delimiting_token)
            } else if let Ok(const_token) = Keyword::from_str(candidate) {
                Self::Const(const_token)
            } else if let Ok(suppoted_type) = DataTypeRaw::from_str(candidate) {
                Self::Type(suppoted_type)
            } else {
                Self::Arbitrary(candidate.to_string())
            },
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub value: TokenValue,
    pub line_number: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at line {}", self.value, self.line_number)
    }
}

pub fn tokenize_statement(input: &str) -> Vec<Token> {
    let mut tokens = Vec::<Token>::new();
    for (line_index, line) in input.split("\n").enumerate() {
        let transparent_split_results = line
            .split_whitespace()
            .filter(|element| !element.is_empty());
        let mut meaningful_split_results = Vec::<String>::new();
        let mut was_eos_encountered = false;
        for pre_token in transparent_split_results {
            let mut current_element: String = "".to_string();
            for character in pre_token.chars() {
                // End tokenization when a statement separator (semicolon) is encountered
                if character == Delimiter::STATEMENT_SEPARATOR {
                    was_eos_encountered = true;
                    break;
                }
                // Recognize delimiters early, as they don't have to be separated by whitespace from other tokens
                if Delimiter::MEANINGFUL_CHARS.contains(&character) {
                    if !current_element.is_empty() {
                        meaningful_split_results.push(current_element.clone());
                    }
                    meaningful_split_results.push(character.to_string());
                    current_element.clear();
                } else {
                    current_element.push(character);
                }
            }
            if !current_element.is_empty() {
                meaningful_split_results.push(current_element);
            }
            if was_eos_encountered {
                break;
            }
        }
        tokens.extend(meaningful_split_results.iter().map(|candidate| Token {
            value: TokenValue::from_str(&candidate).unwrap(),
            line_number: line_index + 1,
        }))
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn tokenization_works_with_create_table() {
        let statement = "CREATE TABLE IF NOT EXISTS test (
            server_id nullable(UINT64),
            hash UINT128 METRIC KEY,
            sent_at TIMESTAMP
        );";

        let detected_tokens = tokenize_statement(&statement);

        let expected_tokens = [
            Token {
                value: TokenValue::Const(Keyword::Create),
                line_number: 1,
            },
            Token {
                value: TokenValue::Const(Keyword::Table),
                line_number: 1,
            },
            Token {
                value: TokenValue::Const(Keyword::If),
                line_number: 1,
            },
            Token {
                value: TokenValue::Const(Keyword::Not),
                line_number: 1,
            },
            Token {
                value: TokenValue::Const(Keyword::Exists),
                line_number: 1,
            },
            Token {
                value: TokenValue::Arbitrary("test".to_string()),
                line_number: 1,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                line_number: 1,
            },
            // New line
            Token {
                value: TokenValue::Arbitrary("server_id".to_string()),
                line_number: 2,
            },
            Token {
                value: TokenValue::Const(Keyword::Nullable),
                line_number: 2,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                line_number: 2,
            },
            Token {
                value: TokenValue::Type(DataTypeRaw::UInt64),
                line_number: 2,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                line_number: 2,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::Comma),
                line_number: 2,
            },
            // New line
            Token {
                value: TokenValue::Arbitrary("hash".to_string()),
                line_number: 3,
            },
            Token {
                value: TokenValue::Type(DataTypeRaw::UInt128),
                line_number: 3,
            },
            Token {
                value: TokenValue::Const(Keyword::Metric),
                line_number: 3,
            },
            Token {
                value: TokenValue::Const(Keyword::Key),
                line_number: 3,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::Comma),
                line_number: 3,
            },
            // New line
            Token {
                value: TokenValue::Arbitrary("sent_at".to_string()),
                line_number: 4,
            },
            Token {
                value: TokenValue::Type(DataTypeRaw::Timestamp),
                line_number: 4,
            },
            // New line
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                line_number: 5,
            },
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
            Token {
                value: TokenValue::Const(Keyword::Create),
                line_number: 1,
            },
            Token {
                value: TokenValue::Const(Keyword::Table),
                line_number: 1,
            },
            Token {
                value: TokenValue::Const(Keyword::If),
                line_number: 1,
            },
            Token {
                value: TokenValue::Const(Keyword::Not),
                line_number: 1,
            },
            Token {
                value: TokenValue::Const(Keyword::Exists),
                line_number: 1,
            },
            Token {
                value: TokenValue::Arbitrary("TEST".to_string()),
                line_number: 1,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                line_number: 1,
            },
            Token {
                value: TokenValue::Arbitrary("serverId".to_string()),
                line_number: 2,
            },
            Token {
                value: TokenValue::Const(Keyword::Nullable),
                line_number: 2,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                line_number: 2,
            },
            Token {
                value: TokenValue::Type(DataTypeRaw::UInt64),
                line_number: 2,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                line_number: 2,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                line_number: 3,
            },
        ];
        assert_eq!(&detected_tokens, &expected_tokens)
    }
}
