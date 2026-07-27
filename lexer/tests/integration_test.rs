use lexer::*;
use pretty_assertions::assert_eq;
use std::collections::VecDeque;
use std::fs;
use std::io::Write;

#[test]
fn test_lexing_failure() {
    let tokenizer = Tokenizer::new();
    let tokens = tokenizer.lex_file(
        &fs::read_to_string("../chuda_programs/lexer_files/lex_test_2.chuda")
            .expect("No file found"),
    );
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
    let tokens = tokenizer.lex_file(
        &fs::read_to_string("../chuda_programs/lexer_files/lex_test_3.chuda")
            .expect("File Not Found"),
    );
    let mut targets: VecDeque<Token> = VecDeque::new();
    targets.push_back(Token::new(1, 1, TokenType::Identifier("Pizza".to_string())));
    targets.push_back(Token::new(1, 6, TokenType::Colon));
    targets.push_back(Token::new(1, 7, TokenType::Int));
    targets.push_back(Token::new(1, 10, TokenType::Assign));
    targets.push_back(Token::new(1, 11, TokenType::Integer(10)));
    targets.push_back(Token::new(1, 13, TokenType::Eof));
    assert_eq!(tokens, targets)
}

#[test]
fn test_lex_multiple() {
    let (tokens, errs) = lexer::lex_files(&vec![
        fs::read_to_string("../chuda_programs/lexer_files/lex_test_3.chuda".to_string())
            .expect("File Not Found"),
        fs::read_to_string("../chuda_programs/lexer_files/all_tokens.chuda".to_string())
            .expect("File Not Found"),
    ]);
    let mut targets: VecDeque<Token> = VecDeque::new();
    targets.push_back(Token::new(1, 1, TokenType::Identifier("Pizza".to_string())));
    targets.push_back(Token::new(1, 6, TokenType::Colon));
    targets.push_back(Token::new(1, 7, TokenType::Int));
    targets.push_back(Token::new(1, 10, TokenType::Assign));
    targets.push_back(Token::new(1, 11, TokenType::Integer(10)));
    targets.push_back(Token::new(1, 13, TokenType::Eof));
    let mut targets2 = VecDeque::new();
    targets2.push_back(Token::new(1, 1, TokenType::LParen));
    targets2.push_back(Token::new(1, 2, TokenType::RParen));
    targets2.push_back(Token::new(1, 3, TokenType::LBrace));
    targets2.push_back(Token::new(1, 4, TokenType::RBrace));
    targets2.push_back(Token::new(1, 5, TokenType::LBracket));
    targets2.push_back(Token::new(1, 6, TokenType::RBracket));
    targets2.push_back(Token::new(1, 7, TokenType::Comma));
    targets2.push_back(Token::new(1, 8, TokenType::Period));
    targets2.push_back(Token::new(1, 9, TokenType::Semicolon));
    targets2.push_back(Token::new(1, 10, TokenType::Colon));
    targets2.push_back(Token::new(1, 11, TokenType::Plus));
    targets2.push_back(Token::new(1, 12, TokenType::Minus));
    targets2.push_back(Token::new(1, 13, TokenType::Mod));
    targets2.push_back(Token::new(1, 14, TokenType::And));
    targets2.push_back(Token::new(1, 15, TokenType::Or));
    targets2.push_back(Token::new(1, 16, TokenType::Underscore));
    targets2.push_back(Token::new(1, 17, TokenType::Times));
    targets2.push_back(Token::new(1, 18, TokenType::HighMultiplication));
    targets2.push_back(Token::new(1, 21, TokenType::Exclamation));
    targets2.push_back(Token::new(1, 22, TokenType::NE));
    targets2.push_back(Token::new(1, 24, TokenType::LAngle));
    targets2.push_back(Token::new(1, 25, TokenType::LE));
    targets2.push_back(Token::new(1, 27, TokenType::RAngle));
    targets2.push_back(Token::new(1, 28, TokenType::GE));
    targets2.push_back(Token::new(1, 30, TokenType::EQ));
    targets2.push_back(Token::new(1, 32, TokenType::Assign));
    targets2.push_back(Token::new(1, 33, TokenType::Divide));
    targets2.push_back(Token::new(
        2,
        1,
        TokenType::String("Hello World".to_string()),
    ));
    targets2.push_back(Token::new(2, 19, TokenType::Character("c".to_string())));
    targets2.push_back(Token::new(2, 22, TokenType::Integer(12345)));
    targets2.push_back(Token::new(2, 27, TokenType::Int));
    targets2.push_back(Token::new(
        2,
        31,
        TokenType::Identifier("chicken_nugget".to_string()),
    ));
    targets2.push_back(Token::new(3, 1, TokenType::Int));
    targets2.push_back(Token::new(3, 5, TokenType::Bool));
    targets2.push_back(Token::new(3, 10, TokenType::True));
    targets2.push_back(Token::new(3, 15, TokenType::False));
    targets2.push_back(Token::new(3, 21, TokenType::While));
    targets2.push_back(Token::new(3, 27, TokenType::Return));
    targets2.push_back(Token::new(3, 34, TokenType::Length));
    targets2.push_back(Token::new(3, 41, TokenType::Use));
    targets2.push_back(Token::new(3, 45, TokenType::If));
    targets2.push_back(Token::new(3, 48, TokenType::Else));
    targets2.push_back(Token::new(
        4,
        6,
        TokenType::Error("Unexpected character: ~".to_string()),
    ));
    let outputs = vec![targets, targets2];
    for (mut result, mut goal) in tokens.into_iter().zip(outputs) {
        dbg!(&result);
        dbg!(&goal);
        assert_eq!(result.len(), goal.len());
        while let Some(t) = result.pop_front()
            && let Some(g) = goal.pop_front()
        {
            assert_eq!(t, g);
        }
    }
    assert!(errs.is_some())
}

#[test]
fn test_lex_all_tokens() {
    let mut outputs = vec![];
    outputs.push("1:1 (".to_string());
    outputs.push("1:2 )".to_string());
    outputs.push("1:3 {".to_string());
    outputs.push("1:4 }".to_string());
    outputs.push("1:5 [".to_string());
    outputs.push("1:6 ]".to_string());
    outputs.push("1:7 ,".to_string());
    outputs.push("1:8 .".to_string());
    outputs.push("1:9 ;".to_string());
    outputs.push("1:10 :".to_string());
    outputs.push("1:11 +".to_string());
    outputs.push("1:12 -".to_string());
    outputs.push("1:13 %".to_string());
    outputs.push("1:14 &".to_string());
    outputs.push("1:15 |".to_string());
    outputs.push("1:16 _".to_string());
    outputs.push("1:17 *".to_string());
    outputs.push("1:18 *>>".to_string());
    outputs.push("1:21 !".to_string());
    outputs.push("1:22 !=".to_string());
    outputs.push("1:24 <".to_string());
    outputs.push("1:25 <=".to_string());
    outputs.push("1:27 >".to_string());
    outputs.push("1:28 >=".to_string());
    outputs.push("1:30 ==".to_string());
    outputs.push("1:32 =".to_string());
    outputs.push("1:33 /".to_string());
    outputs.push("2:1 string Hello World".to_string());
    outputs.push("2:19 character c".to_string());
    outputs.push("2:22 integer 12345".to_string());
    outputs.push("2:27 int".to_string());
    outputs.push("2:31 id chicken_nugget".to_string());
    outputs.push("3:1 int".to_string());
    outputs.push("3:5 bool".to_string());
    outputs.push("3:10 true".to_string());
    outputs.push("3:15 false".to_string());
    outputs.push("3:21 while".to_string());
    outputs.push("3:27 return".to_string());
    outputs.push("3:34 length".to_string());
    outputs.push("3:41 use".to_string());
    outputs.push("3:45 if".to_string());
    outputs.push("3:48 else".to_string());
    outputs.push("4:6 error: Unexpected character: ~".to_string());
    let tokenizer = Tokenizer::new();
    let source_text =
        fs::read_to_string("../chuda_programs/lexer_files/all_tokens.chuda").expect("Read Error");
    let mut tokens = tokenizer.lex_file(&source_text);
    dbg!(&tokens);
    dbg!(&outputs);
    let mut buffer = vec![];
    while let Some(item) = tokens.pop_front() {
        writeln!(buffer, "{}", item).expect("Write Failure")
    }
    dbg!(&buffer);
    let mut output_string = outputs.join("\n");
    output_string.push('\n');
    assert_eq!(
        String::from_utf8(buffer).expect("Invalid utf-8"),
        output_string
    );
}
