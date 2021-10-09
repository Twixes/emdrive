use std::str::FromStr;

use crate::construct::components::DataTypeRaw;
use std::fmt::{self, Debug};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Delimiter {
    Comma,
    ParenthesisOpening,
    ParenthesisClosing,
}

impl Delimiter {
    /// Delimiting characters that affect statement meaning. Each one is a Delimiter variant.
    const MEANINGFUL_CHARS: &'static [char] = &[',', '(', ')'];
    const STATEMENT_SEPARATOR: char = ';';
    const STRING_MARKER: char = '\'';
    const ESCAPE_CHARACTER: char = '\\';
}

impl fmt::Display for Delimiter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Comma => "comma `,`",
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
    Null,
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
                Keyword::Null => "NULL",
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
            "null" => Ok(Self::Null),
            _ => Err(format!("`{}` does not refer to a const token", candidate)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenValue {
    Delimiting(Delimiter),
    Const(Keyword),
    Type(DataTypeRaw),
    String(String),
    Arbitrary(String),
}

impl fmt::Display for TokenValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Delimiting(value) => fmt::Display::fmt(&value, f),
            Self::Const(value) => fmt::Display::fmt(&value, f),
            Self::Type(value) => value.fmt(f),
            Self::String(value) => write!(f, "string `\"{}\"`", value),
            Self::Arbitrary(value) => write!(f, "arbitrary `{}`", value),
        }
    }
}

impl FromStr for TokenValue {
    type Err = ();

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        if let Ok(delimiter) = Delimiter::from_str(candidate) {
            Ok(Self::Delimiting(delimiter))
        } else if let Ok(keyword) = Keyword::from_str(candidate) {
            Ok(Self::Const(keyword))
        } else if let Ok(data_type_raw) = DataTypeRaw::from_str(candidate) {
            Ok(Self::Type(data_type_raw))
        } else {
            let mut candidate_chars = candidate.chars();
            if let (Some(Delimiter::STRING_MARKER), Some(Delimiter::STRING_MARKER)) =
                (candidate_chars.next(), candidate_chars.next_back())
            {
                // We only need to use as_str() here as the first and last chars have been consumed by nth()s
                Ok(Self::String(candidate_chars.as_str().to_string()))
            } else {
                Ok(Self::Arbitrary(candidate.to_string()))
            }
        }
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
    for (line_index, line) in input.lines().enumerate() {
        let mut token_candidates = Vec::<String>::new();
        let mut current_candidate: String = "".to_string();
        let mut is_current_character_escaped = false;
        let mut is_current_character_inside_string = false;
        for character in line.chars() {
            // Act upon tokenization-level semantics, but only if the current character is not escaped with a backslash
            if !is_current_character_escaped {
                // Detect if the next character is escaped
                if character == Delimiter::ESCAPE_CHARACTER {
                    is_current_character_escaped = true;
                    continue;
                }
                // Detect if this character starts/ends a string
                if character == Delimiter::STRING_MARKER {
                    current_candidate.push(character);
                    if is_current_character_inside_string {
                        token_candidates.push(current_candidate.clone());
                        current_candidate.clear();
                        is_current_character_inside_string = false;
                    } else {
                        is_current_character_inside_string = true;
                    }
                    continue;
                }
                if !is_current_character_inside_string {
                    // End tokenization when a statement separator (semicolon) is encountered
                    if character == Delimiter::STATEMENT_SEPARATOR {
                        break;
                    }
                    // Recognize delimiters earlier, as they don't have to be separated by whitespace from other tokens
                    if Delimiter::MEANINGFUL_CHARS.contains(&character) {
                        if !current_candidate.is_empty() {
                            token_candidates.push(current_candidate.clone());
                            current_candidate.clear();
                        }
                        token_candidates.push(character.to_string());
                        continue;
                    }
                    // Break up non-delimiter tokens on whitespace
                    if character.is_ascii_whitespace() {
                        if !current_candidate.is_empty() {
                            token_candidates.push(current_candidate.clone());
                            current_candidate.clear();
                        }
                        continue;
                    }
                }
            } else {
                // Reset escape status for the next character
                is_current_character_escaped = false;
            }
            // The default case for a character is just being appended to the working token candidate string
            current_candidate.push(character);
        }
        // Add line remainded to token candidates
        if !current_candidate.is_empty() {
            token_candidates.push(current_candidate);
        }
        // Process token candidates found on this line
        tokens.extend(token_candidates.iter().map(|candidate| Token {
            value: TokenValue::from_str(candidate).unwrap(),
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

    #[test]
    fn tokenization_supports_various_strings() {
        let statement = "INSERT INTO test
            (foo, bar, baz)
            VALUES ('123', '   x ', 'The \\'Moon\\'')";

        let detected_tokens = tokenize_statement(&statement);

        let expected_tokens = [
            Token {
                value: TokenValue::Const(Keyword::Insert),
                line_number: 1,
            },
            Token {
                value: TokenValue::Const(Keyword::Into),
                line_number: 1,
            },
            Token {
                value: TokenValue::Arbitrary("test".to_string()),
                line_number: 1,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                line_number: 2,
            },
            Token {
                value: TokenValue::Arbitrary("foo".to_string()),
                line_number: 2,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::Comma),
                line_number: 2,
            },
            Token {
                value: TokenValue::Arbitrary("bar".to_string()),
                line_number: 2,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::Comma),
                line_number: 2,
            },
            Token {
                value: TokenValue::Arbitrary("baz".to_string()),
                line_number: 2,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                line_number: 2,
            },
            Token {
                value: TokenValue::Const(Keyword::Values),
                line_number: 3,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisOpening),
                line_number: 3,
            },
            Token {
                value: TokenValue::String("123".to_string()),
                line_number: 3,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::Comma),
                line_number: 3,
            },
            Token {
                value: TokenValue::String("   x ".to_string()),
                line_number: 3,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::Comma),
                line_number: 3,
            },
            Token {
                value: TokenValue::String("The \'Moon\'".to_string()),
                line_number: 3,
            },
            Token {
                value: TokenValue::Delimiting(Delimiter::ParenthesisClosing),
                line_number: 3,
            },
        ];
        assert_eq!(&detected_tokens, &expected_tokens)
    }
}
