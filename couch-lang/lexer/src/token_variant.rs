use std::fmt::Display;

#[derive(PartialEq, Debug)]
pub enum TokenVariant {
    LetKeyword,
    MutKeyword,
    FnKeyword,
    ReturnKeyword,
    Identifier,
    LParenthesis,
    RParenthesis,
    LBrace,
    RBrace,
    Equal,
    PlusEqual,
    Plus,
    MinusEqual,
    Minus,
    AsteriskEqual,
    Asterisk,
    SlashEqual,
    Slash,
    Semicolon,
    Integer,
    Float,
    Error,
    Exclamation,
    ExclamationEqual,
    DoubleEqual,
}

impl Display for TokenVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TokenVariant::LetKeyword => "LetKeyword",
            TokenVariant::MutKeyword => "MutKeyword",
            TokenVariant::FnKeyword => "FnKeyword",
            TokenVariant::ReturnKeyword => "ReturnKeyword",
            TokenVariant::Identifier => "Identifier",
            TokenVariant::LParenthesis => "LeftParenthesis",
            TokenVariant::RParenthesis => "RightParenthesis",
            TokenVariant::LBrace => "LeftBrace",
            TokenVariant::RBrace => "RightBrace",
            TokenVariant::Equal => "Equal",
            TokenVariant::PlusEqual => "PlusEqual",
            TokenVariant::Plus => "Plus",
            TokenVariant::MinusEqual => "MinusEqual",
            TokenVariant::Minus => "Minus",
            TokenVariant::AsteriskEqual => "AsteriskEqual",
            TokenVariant::Asterisk => "Asterisk",
            TokenVariant::SlashEqual => "SlashEqual",
            TokenVariant::Slash => "Slash",
            TokenVariant::Semicolon => "Semicolon",
            TokenVariant::Integer => "Integer",
            TokenVariant::Float => "Float",
            TokenVariant::Error => "Error",
            TokenVariant::Exclamation => "Exclamation",
            TokenVariant::ExclamationEqual => "ExclamationEqual",
            TokenVariant::DoubleEqual => "DoubleEqual",
        })
    }
}
