use std::{collections::HashMap, str::FromStr};

const TOKEN_DELIMITERS_TRANSPARENT: &[char] = &[' ', '\t'];

enum DelimitingToken {
    Comma,
    Semicolon,
    SingleQuote,
    DoubleQuote,
    ParenthesisOpening,
    ParenthesisClosing
}

enum ConstToken {
    CREATE,
    TABLE,
    IF,
    NOT,
    EXISTS,
    OPTIONAL,
    POSITION,
    BY
}

pub enum SQLToken {
    Const(ConstToken),
    Type(SupportedType),
    Arbitrary(String)
}

#[derive(Debug)]
enum SupportedType {
    U64,
    U128,
    Timestamp
}

#[derive(Debug)]
struct ColumnDefinition {
    name: String,
    value_type: SupportedType,
    is_optional: bool
}
/*
fn parse_token_candidate(candidate: &str) -> SQLToken {
    if let Ok(const_token) = ConstToken::from_str(candidate) {
        return SQLToken::Const(const_token)
    }
    if let Ok(type_token) = SupportedType::from_str(candidate) {
        return SQLToken::Type(type_token)
    }
    return SQLToken::Arbitrary(candidate.to_string())
}

pub fn tokenize(input: &str) -> Vec<SQLToken> {
    let transparent_split_pass = input.split(TOKEN_DELIMITERS_TRANSPARENT).filter(|element| !element.is_empty());
    let meaningful_split_pass = transparent_split_pass.flat_map(|raw_token|  parse_token_candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenization_works_with_create_table() {
        let statement = "CREATE TABLE IF NOT EXISTS 'test' (
            server_id optional(u64),
            channel_id optional(u64),
            user_id optional(u64),
            attachment_id u64,
            message_id u64,
            hash u128,
            sent_at timestamp
        )
        POSITION BY hash;";

        let tokens: Vec<SQLToken> = 
    }
}
*/
