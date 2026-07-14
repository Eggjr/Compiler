use lexer::*;
use std::collections::VecDeque;

#[test]
fn test_lexing_failure() {
    let tokenizer = Tokenizer::new();
    let tokens = tokenizer
        .lex_file("../eta_programs/lexer_files/lex_test_2.eta")
        .expect("Lexing Failure");
    let mut targets = VecDeque::new();

    targets.push_back(Token::new(1, 1, TokenType::Identifier("x".to_string())));
    targets.push_back(Token::new(1, 2, TokenType::Colon));
    targets.push_back(Token::new(1, 3, TokenType::Bool));
    targets.push_back(Token::new(1, 8, TokenType::Assign));
    targets.push_back(Token::new(1, 10, TokenType::Integer(4)));
    targets.push_back(Token::new(1, 11, TokenType::Identifier("all".to_string())));
    targets.push_back(Token::new(2, 1, TokenType::Identifier("x".to_string())));
    targets.push_back(Token::new(2, 3, TokenType::Assign));
    targets.push_back(Token::new(
        2,
        5,
        TokenType::Error("No Character Given".to_string()),
    ));
    assert_eq!(tokens, targets);
}

#[test]
fn test_lexing_simple() {
    let tokenizer = Tokenizer::new();
    let tokens = tokenizer
        .lex_file("../eta_programs/lexer_files/lex_test_3.eta")
        .expect("Lexing Failure");
    let mut targets: VecDeque<Token> = VecDeque::new();
    targets.push_back(Token::new(1, 1, TokenType::Identifier("Pizza".to_string())));
    targets.push_back(Token::new(1, 6, TokenType::Colon));
    targets.push_back(Token::new(1, 7, TokenType::Int));
    targets.push_back(Token::new(1, 10, TokenType::Assign));
    targets.push_back(Token::new(1, 11, TokenType::Integer(10)));
    assert_eq!(tokens, targets)
}
