use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Identifier(String),
    Integer(u64),
    Character(String),
    String(String),
    LParen,             // (
    RParen,             // )
    LBracket,           // [
    RBracket,           // ]
    LBrace,             // {
    RBrace,             // }
    LAngle,             // <
    RAngle,             // >
    Semicolon,          // ;
    Colon,              // :
    Plus,               // +
    Times,              // *
    HighMultiplication, // *>>
    Minus,              // -
    Divide,             // /
    Mod,                // %
    EQ,                 // ==
    NE,                 // !=
    LE,                 // <=
    GE,                 // >=
    Assign,             // =
    And,                // &
    Or,                 // |
    Exclamation,        // !
    Period,             // .
    Comma,              // ,
    Underscore,         // _
    Int,
    Bool,
    True,
    False,
    While,
    Use,
    If,
    Else,
    Return,
    Length,
    Error(String),
    Eof                 // End of File
}

impl fmt::Display for TokenType {
    /// String representation of token_type to be printed to file as defined in spec
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let to_write = match self {
            TokenType::Identifier(id) => format!("id {}", id),
            TokenType::Integer(val) => format!("integer {}", val),
            TokenType::Character(c) => format!("character {}", c),
            TokenType::String(literal) => format!("string {}", literal),
            TokenType::LParen => String::from("("),
            TokenType::RParen => String::from(")"),
            TokenType::LBracket => String::from("["),
            TokenType::RBracket => String::from("]"),
            TokenType::LBrace => String::from("{"),
            TokenType::RBrace => String::from("}"),
            TokenType::LAngle => String::from("<"),
            TokenType::RAngle => String::from(">"),
            TokenType::Semicolon => String::from(";"),
            TokenType::Colon => String::from(":"),
            TokenType::Plus => String::from("+"),
            TokenType::Times => String::from("*"),
            TokenType::HighMultiplication => String::from("*>>"),
            TokenType::Minus => String::from("-"),
            TokenType::Divide => String::from("/"),
            TokenType::Mod => String::from("%"),
            TokenType::EQ => String::from("=="),
            TokenType::NE => String::from("!="),
            TokenType::LE => String::from("<="),
            TokenType::GE => String::from(">="),
            TokenType::Assign => String::from("="),
            TokenType::And => String::from("&"),
            TokenType::Or => String::from("|"),
            TokenType::Exclamation => String::from("!"),
            TokenType::Period => String::from("."),
            TokenType::Comma => String::from(","),
            TokenType::Underscore => String::from("_"),
            TokenType::Int => String::from("int"),
            TokenType::Bool => String::from("bool"),
            TokenType::True => String::from("true"),
            TokenType::False => String::from("false"),
            TokenType::While => String::from("while"),
            TokenType::Use => String::from("use"),
            TokenType::If => String::from("if"),
            TokenType::Else => String::from("else"),
            TokenType::Return => String::from("return"),
            TokenType::Length => String::from("length"),
            TokenType::Error(msg) => format!("error: {}", msg),
            TokenType::Eof => String::from("")
        };
        write!(f, "{}", to_write)
    }
}
