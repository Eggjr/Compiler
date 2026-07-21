use lexer::*;
use pretty_assertions::assert_eq;
use std::collections::VecDeque;

#[test]
fn test_lexing_failure() {
    let tokenizer = Tokenizer::new();
    let tokens = tokenizer
        .lex_file("../chuda_programs/lexer_files/lex_test_2.chuda")
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
        .lex_file("../chuda_programs/lexer_files/lex_test_3.chuda")
        .expect("Lexing Failure");
    let mut targets: VecDeque<Token> = VecDeque::new();
    targets.push_back(Token::new(1, 1, TokenType::Identifier("Pizza".to_string())));
    targets.push_back(Token::new(1, 6, TokenType::Colon));
    targets.push_back(Token::new(1, 7, TokenType::Int));
    targets.push_back(Token::new(1, 10, TokenType::Assign));
    targets.push_back(Token::new(1, 11, TokenType::Integer(10)));
    assert_eq!(tokens, targets)
}

#[test]
fn test_fake_file() {
    let tokenizer = Tokenizer::new();
    assert!(tokenizer.lex_file("NOT A REAL FILE LOL.chuda").is_err());
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
    let mut tokens = tokenizer
        .lex_file("../chuda_programs/lexer_files/all_tokens.chuda")
        .expect("Lexing failed");
    dbg!(&tokens);
    dbg!(&outputs);
    let mut buffer = vec![];
    lexer::write_tokens(&mut tokens, &mut buffer).expect("Write Failure");
    dbg!(&buffer);
    let mut output_string = outputs.join("\n");
    output_string.push('\n');
    assert_eq!(
        String::from_utf8(buffer).expect("Invalid utf-8"),
        output_string
    );
}
