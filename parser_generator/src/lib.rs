use lexer::Token;
use lexer::TokenType;
use std::convert;
use std::file::{BufReader, BufRead};
use std::fs::File;

#[derive(Debug, PartialEq, Eq, Hash)]
struct Production{
    index : u32,
    head : NonTerminal,
    body : Vec<Symbol>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Terminal{
    Identifier,
    Integer,
    Character,
    String,
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
    Empty,
    Eof                 // End of File
}

impl convert::TryFrom<Token> for Terminal{
    type Error = String;
    fn try_from(item : Token) -> Result<Self, Self::Error>{
        match item.token_type(){
            TokenType::Identifier(_) => Ok(Terminal::Identifier),
            TokenType::Integer(_) => Ok(Terminal::Integer),
            TokenType::Character(_) => Ok(Terminal::Character),
            TokenType::String(_) => Ok(Terminal::String),
            TokenType::LParen => Ok(Terminal::LParen),
            TokenType::RParen => Ok(Terminal::RParen),
            TokenType::LBracket => Ok(Terminal::LBracket),
            TokenType::RBracket => Ok(Terminal::RBracket),
            TokenType::LBrace => Ok(Terminal::LBrace),
            TokenType::RBrace => Ok(Terminal::RBrace),
            TokenType::LAngle => Ok(Terminal::LAngle),
            TokenType::RAngle => Ok(Terminal::RAngle),
            TokenType::Semicolon => Ok(Terminal::Semicolon),
            TokenType::Colon => Ok(Terminal::Colon),
            TokenType::Plus => Ok(Terminal::Plus),
            TokenType::Times => Ok(Terminal::Times),
            TokenType::HighMultiplication => Ok(Terminal::HighMultiplication),
            TokenType::Minus => Ok(Terminal::Minus),
            TokenType::Divide => Ok(Terminal::Divide),
            TokenType::Mod => Ok(Terminal::Mod),
            TokenType::EQ => Ok(Terminal::EQ),
            TokenType::NE => Ok(Terminal::NE),
            TokenType::LE => Ok(Terminal::LE),
            TokenType::GE => Ok(Terminal::GE),
            TokenType::Assign => Ok(Terminal::Assign),
            TokenType::And => Ok(Terminal::And),
            TokenType::Or => Ok(Terminal::Or),
            TokenType::Exclamation => Ok(Terminal::Exclamation),
            TokenType::Period => Ok(Terminal::Period),
            TokenType::Comma => Ok(Terminal::Comma),
            TokenType::Underscore => Ok(Terminal::Underscore),
            TokenType::Int => Ok(Terminal::Int),
            TokenType::Bool => Ok(Terminal::Bool),
            TokenType::True => Ok(Terminal::True),
            TokenType::False => Ok(Terminal::False),
            TokenType::While => Ok(Terminal::While),
            TokenType::Use => Ok(Terminal::Use),
            TokenType::If => Ok(Terminal::If),
            TokenType::Else => Ok(Terminal::Else),
            TokenType::Return => Ok(Terminal::Return),
            TokenType::Length => Ok(Terminal::Length),
            TokenType::Eof => Ok(Terminal::Eof),
            TokenType::Error(msg) => Err(format!("Got Unexpected Error Token in Parsing: {}" ,msg)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum NonTerminal{
    File,
    ProgramFile,
    InterfaceFile,
    FunctionDeclarationList,
    UseList,
    GlobalDeclarationList,
    GlobalDeclaration,
    ScalarTypeID,
    GlobalInit,
    FunctionDeclaration,
    ParamListOpt,
    ParamList,
    ReturnType,
    TypeList,
    TypedIDList,
    TypedId,
    Type,
    Primitive,
    ArrayDeclaration,
    Block,
    StatementList,
    Statement,
    VarDeclaration,
    ElseOpt,
    LastStatementOpt,
    AssignableList,
    Assignable,
    Target,
    FunctionCall,
    ExprListOpt,
    ExprList,
    ExprOpt,
    Condition,
    Conjuction,
    Equality,
    EqualityOp,
    Relation,
    RelationOp,
    Expr,
    ExprOp,
    Term,
    TermOp,
    Unary,
    UnaryOp,
    Factor,
    Atom,
    ArrayLiteral,
    OptSemi,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Symbol{
    NT(NonTerminal),
    T(Terminal),
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Production{
    lhs : Symbol,
    rhs : Vec<Symbol>
}

fn read_grammar(file_name : &str) -> Vec<Vec<Symbol>>{
    //shift, reduce, accept are columns
    let mut action_state_table = Vec![Vec::new(), Vec::new(), Vec::new()];
    let file = File::opem(&file_name);
    let reader = BufReader::new(file);

    return action_state_table
}