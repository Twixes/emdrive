use std::str::FromStr;

#[derive(Debug, PartialEq)]
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
                "\"{}\" does not refer to a meaningful delimiter",
                candidate
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ConstToken {
    Create,
    Table,
    If,
    Not,
    Exists,
    Optional,
    Metric,
    Key,
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
            "optional" => Ok(Self::Optional),
            "metric" => Ok(Self::Metric),
            "key" => Ok(Self::Key),
            _ => Err(format!("\"{}\" does not refer to a const token", candidate)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SupportedType {
    U8,
    U16,
    U32,
    U64,
    U128,
    Timestamp,
    String,
}

impl FromStr for SupportedType {
    type Err = String;

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        match candidate.to_lowercase().as_str() {
            "u64" => Ok(Self::U64),
            "u128" => Ok(Self::U128),
            "timestamp" => Ok(Self::Timestamp),
            _ => Err(format!(
                "\"{}\" does not refer to a supported type",
                candidate
            )),
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum Token {
    Delimiting(Delimiter),
    Const(ConstToken),
    Type(SupportedType),
    Arbitrary(String),
}

impl FromStr for Token {
    type Err = ();

    fn from_str(candidate: &str) -> std::result::Result<Self, Self::Err> {
        Ok(
            if let Ok(delimiting_token) = Delimiter::from_str(candidate) {
                Self::Delimiting(delimiting_token)
            } else if let Ok(const_token) = ConstToken::from_str(candidate) {
                Self::Const(const_token)
            } else if let Ok(suppoted_type) = SupportedType::from_str(candidate) {
                Self::Type(suppoted_type)
            } else {
                Self::Arbitrary(candidate.to_string())
            },
        )
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let transparent_split_pass = input
        .split(Delimiter::TRANSPARENT_CHARS)
        .filter(|element| !element.is_empty());
    let mut meaningful_split_pass = Vec::<String>::new();
    for part in transparent_split_pass {
        let mut current_element: String = "".to_string();
        for character in part.chars() {
            if Delimiter::MEANINGFUL_CHARS.contains(&character) {
                if !current_element.is_empty() {
                    meaningful_split_pass.push(current_element.clone());
                }
                meaningful_split_pass.push(character.to_string());
                current_element.clear();
            } else {
                current_element.push(character);
            }
        }
        if !current_element.is_empty() {
            meaningful_split_pass.push(current_element);
        }
    }
    println!("Meaningful: {:?}", meaningful_split_pass);
    let tokens: Vec<Token> = meaningful_split_pass
        .iter()
        .map(|candidate| Token::from_str(&candidate).unwrap())
        .collect();
    Ok(tokens)
}
struct ColumnDefinition {
    name: String,
    value_type: SupportedType,
    is_optional: bool,
}

////////////////////////

struct CreateCommand;

struct DropCommand;
struct InsertCommand;
struct SelectCommand;

enum Command {
    Create(CreateCommand),
    Drop(DropCommand),
    Insert(InsertCommand),
}

////////////////////////

#[cfg(test)]
mod tests {
    use crate::utils::vec_eq_exact;

    use super::*;

    #[test]
    fn tokenization_works_with_create_table() {
        let statement = "CREATE TABLE IF NOT EXISTS 'test' (
            server_id optional(u64),
            hash u128 METRIC KEY,
            sent_at timestamp
        );";

        let detected_tokens: Vec<Token> = tokenize(&statement).unwrap();

        let expected_tokens: Vec<Token> = vec![
            Token::Const(ConstToken::Create),
            Token::Const(ConstToken::Table),
            Token::Const(ConstToken::If),
            Token::Const(ConstToken::Not),
            Token::Const(ConstToken::Exists),
            Token::Delimiting(Delimiter::SingleQuote),
            Token::Arbitrary("test".to_string()),
            Token::Delimiting(Delimiter::SingleQuote),
            Token::Delimiting(Delimiter::ParenthesisOpening),
            // New line
            Token::Arbitrary("server_id".to_string()),
            Token::Const(ConstToken::Optional),
            Token::Delimiting(Delimiter::ParenthesisOpening),
            Token::Type(SupportedType::U64),
            Token::Delimiting(Delimiter::ParenthesisClosing),
            Token::Delimiting(Delimiter::Comma),
            // New line
            Token::Arbitrary("hash".to_string()),
            Token::Type(SupportedType::U128),
            Token::Const(ConstToken::Metric),
            Token::Const(ConstToken::Key),
            Token::Delimiting(Delimiter::Comma),
            // New line
            Token::Arbitrary("sent_at".to_string()),
            Token::Type(SupportedType::Timestamp),
            // New line
            Token::Delimiting(Delimiter::ParenthesisClosing),
            Token::Delimiting(Delimiter::Semicolon),
        ];
        assert!(vec_eq_exact(&detected_tokens, &expected_tokens))
    }

    #[test]
    fn tokenization_is_case_sensitive_and_insensitive_properly() {
        let statement = "CREATE table If nOT exists 'TEST' (
            serverId optionAl(U64)
        )";

        let detected_tokens: Vec<Token> = tokenize(&statement).unwrap();

        let expected_tokens: Vec<Token> = vec![
            Token::Const(ConstToken::Create),
            Token::Const(ConstToken::Table),
            Token::Const(ConstToken::If),
            Token::Const(ConstToken::Not),
            Token::Const(ConstToken::Exists),
            Token::Delimiting(Delimiter::SingleQuote),
            Token::Arbitrary("TEST".to_string()),
            Token::Delimiting(Delimiter::SingleQuote),
            Token::Delimiting(Delimiter::ParenthesisOpening),
            Token::Arbitrary("serverId".to_string()),
            Token::Const(ConstToken::Optional),
            Token::Delimiting(Delimiter::ParenthesisOpening),
            Token::Type(SupportedType::U64),
            Token::Delimiting(Delimiter::ParenthesisClosing),
            Token::Delimiting(Delimiter::ParenthesisClosing),
        ];
        assert!(vec_eq_exact(&detected_tokens, &expected_tokens))
    }
}

/*
CREATE TABLE IF NOT EXISTS 'test' (
            server_id optional(u64),
            channel_id optional(u64),
            user_id optional(u64),
            attachment_id u64,
            message_id u64,
            hash u128,
            sent_at timestamp
        )
        POSITION BY hash;
*/
