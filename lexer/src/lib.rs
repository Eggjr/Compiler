pub mod token;
pub mod token_type;

pub mod lexer {
    use crate::lexer::LexerError::{
        IOError, IntegerTooLarge, InvalidFilesError, UnclosedCharacter, UnclosedString,
        UnexpectedCharacterConstant, UnexpectedEscapeSequence, UnexpectedHexEscape,
    };
    use crate::token::token::Token;
    use crate::token_type::token_type::TokenType;
    use std::collections::VecDeque;
    use std::fs;
    use std::iter::Peekable;
    use std::path::PathBuf;
    use std::str::Chars;

    #[derive(Debug)]
    pub enum LexerError {
        InvalidFilesError(Vec<String>),
        IOError(String, std::io::Error),
        UnexpectedCharacterConstant(String),
        NoMoreTokens(String),
        UnclosedString(String),
        UnexpectedHexEscape(String),
        UnexpectedEscapeSequence(String),
        UnclosedCharacter(String),
        IntegerTooLarge(String),
    }

    fn verify_files(source_files: &Vec<String>) -> Result<(), LexerError> {
        let invalids: Vec<String> = source_files
            .iter()
            .filter(|file| !file.ends_with(".eta") && !file.ends_with(".eti"))
            .cloned()
            .collect();
        if !invalids.is_empty() {
            return Err(InvalidFilesError(invalids));
        }
        return Ok(());
    }

    fn construct_paths(source_files: &Vec<String>, path: PathBuf) -> Vec<PathBuf> {
        return source_files
            .iter()
            .map(|file_name| (path.join(file_name)).with_extension("lexed"))
            .collect();
    }

    #[derive(Debug, Clone)]
    struct Tokenizer {
        line: usize,
        column: usize,
        input_file: String,
        tokens: VecDeque<Token>,
        index: usize,
        input_text: String,
    }

    impl Tokenizer {
        fn build(input_file: String) -> Result<Tokenizer, LexerError> {
            let file_contents = match fs::read_to_string(input_file) {
                Ok(file_contents) => file_contents,
                Err(e) => {
                    return Err(IOError(input_file.to_owned(), e));
                }
            };
            let stream = file_contents.chars().peekable();
            Ok(Tokenizer {
                line: 1,
                column: 1,
                stream,
                input_file,
                tokens: VecDeque::new(),
                index: 0,
                input_text: file_contents,
            })
        }

        fn consume(&mut self) -> Option<char> {
            if let Some(c) = self.stream.next() {
                self.index += c.len_utf8();
                match c {
                    '\n' => {
                        self.line += 1;
                        self.column = 1;
                        return None;
                    }
                    '\r' => {
                        if let Some(newline) = self.stream.peek() {
                            if *newline == '\n' {
                                self.stream.next();
                                self.index += 1
                            }
                        }
                        self.line += 1;
                        self.column = 1;
                        return None;
                    }
                    other => {
                        self.column += 1;
                        return Some(other);
                    }
                };
            }
            None
        }

        fn remove_whitespace(&mut self) -> () {
            while let Some(c) = self.stream.peek() {
                match c {
                    '\n' | '\t' | ' ' | '\r' => {
                        self.consume();
                        ()
                    }
                    _nonwhitespace => return (),
                }
            }
            ()
        }

        fn consume_comment(&mut self) -> () {
            while let Some(c) = self.consume() {
                if c == '\n' {
                    break;
                }
            }
        }

        fn lex_slash(&mut self) -> () {
            self.consume();
            match self.stream.peek() {
                Some('/') => self.consume_comment(),
                _nonslash => self.push_token(TokenType::Divide),
            }
            ()
        }

        fn hex_to_char(&mut self) -> Result<char, LexerError> {
            let mut res: String = String::from("");
            match self.consume() {
                Some('{') => (),
                _ => {
                    return Err(UnexpectedHexEscape(format!(
                        "Expected {{ to begin Unicode Character hex value hex at {}:{}",
                        self.line - 1,
                        self.column
                    )));
                }
            }
            let mut digits = 0;
            while let Some(c) = self.consume()
                && digits < 6
            {
                if c.is_ascii_hexdigit() {
                    res.push(c);
                    digits += 1;
                } else if c == '}' {
                    let u32_rep = match u32::from_str_radix(res.as_str(), 16) {
                        Ok(u) => u,
                        Err(e) => return Err(UnexpectedHexEscape(format!("{}", e))),
                    };
                    match char::from_u32(u32_rep) {
                        Some(c) => return Ok(c),
                        None => {
                            return Err(UnexpectedHexEscape(format!(
                                "Hexadecimal number {} could not be parsed to valid unicode",
                                res
                            )));
                        }
                    }
                } else {
                    return Err(UnexpectedHexEscape(format!(
                        "Expected valid hexadecimal character but got {}",
                        c
                    )));
                }
            }
            Err(UnexpectedHexEscape(format!(
                "Expected }} to end Unicode Character value hex at {}:{}",
                self.line - 1,
                self.column
            )))
        }

        fn lex_character(&mut self) -> Result<(), LexerError> {
            let val = match self.consume() {
                Some('\\') => match self.consume() {
                    Some('x') => self.hex_to_char()?,
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('\'') => '\'',
                    Some('\"') => '\"',
                    Some('\\') => '\\',
                    Some(c) => {
                        return Err(UnexpectedEscapeSequence(format!(
                            "Unexpected Escape Sequence found: \\{}",
                            c
                        )));
                    }
                    None => {
                        return Err(UnexpectedEscapeSequence(
                            "Character did not terminate. Expected <char>\' but got nothing"
                                .to_string(),
                        ));
                    }
                },
                Some('\'') => {
                    return Err(UnexpectedCharacterConstant(format!("No Character Given")));
                }
                Some(c) => c,
                None => {
                    return Err(UnclosedCharacter(format!(
                        "Expected character literal to be closed with \'"
                    )));
                }
            };
            if let Some(c) = self.consume() {
                match c {
                    '\'' => {
                        self.push_token(TokenType::Character(val));
                        return Ok(());
                    }
                    c => {
                        return Err(UnclosedCharacter(format!(
                            "Expected character literal to be closed with \' but got {}",
                            c
                        )));
                    }
                }
            }
            Ok(())
        }

        fn lex_string(&mut self) -> Result<(), LexerError> {
            let mut contents = String::new();
            let start_col = self.column - 1;
            while let Some(c) = self.consume() {
                match c {
                    '\"' => {
                        self.tokens.push_back(Token::new(
                            self.line,
                            start_col,
                            TokenType::String(contents),
                        ));
                        return Ok(());
                    }
                    '\\' => match self.stream.peek() {
                        Some('x') => contents.push(self.hex_to_char()?),
                        _ => (),
                    },
                    _ => {
                        contents.push(c);
                    }
                }
            }
            return Err(UnclosedString(format!(
                "Expected \" to close String at {}:{}",
                self.line, self.column
            )));
        }

        fn lex_langle(&mut self) {
            match self.stream.peek() {
                Some('=') => {
                    self.push_token(TokenType::LE);
                    self.consume();
                }
                _ => self.push_token(TokenType::LAngle),
            }
        }

        fn lex_rangle(&mut self) {
            match self.stream.peek() {
                Some('=') => {
                    self.push_token(TokenType::GE);
                    self.consume();
                }
                _ => self.push_token(TokenType::RAngle),
            }
        }

        fn lex_exclamation(&mut self) {
            match self.stream.peek() {
                Some('=') => {
                    self.push_token(TokenType::NE);
                    self.consume();
                }
                _ => self.push_token(TokenType::Exclamation),
            }
        }

        fn lex_equal(&mut self) {
            match self.stream.peek() {
                Some('=') => {
                    self.push_token(TokenType::EQ);
                    self.consume();
                }
                _ => self.push_token(TokenType::Assign),
            }
        }

        fn lex_integer(&mut self) -> Result<(), LexerError> {
            let first = self.index - 1;
            while let Some(c) = self.stream.peek() {
                if !c.is_ascii_digit() {
                    break;
                } else {
                    self.consume();
                }
            }
            //Handle i64 bounds in the parser
            self.push_token(TokenType::Integer(
                match self.input_text[first..self.index].parse::<u64>() {
                    Ok(val) => val,
                    Err(e) => return Err(IntegerTooLarge(format!("Trouble Parsing int: {:?}", e))),
                },
            ));
            Ok(())
        }

        fn lex_key_or_identifier(&mut self) {
            let first = self.index - 1;
            while let Some(c) = self.stream.peek() {
                if !(c.is_ascii_alphanumeric() || *c == '_') {
                    break;
                } else {
                    self.consume();
                }
            }
            let ttype = match &self.input_text[first..self.index] {
                "int" => TokenType::Int,
                "bool" => TokenType::Bool,
                "while" => TokenType::While,
                "use" => TokenType::Use,
                "return" => TokenType::Return,
                "length" => TokenType::Length,
                "true" => TokenType::True,
                "false" => TokenType::False,
                "if" => TokenType::If,
                "else" => TokenType::Else,
                identifier => TokenType::Identifier(identifier.to_string()),
            };
            self.push_token(ttype);
        }

        fn push_token(&mut self, token_type: TokenType) -> () {
            self.tokens
                .push_back(Token::new(self.line, self.column - 1, token_type));
        }

        fn lex_file(mut self) -> Result<Option<VecDeque<Token>>, LexerError> {
            while let Some(c) = self.consume() {
                self.remove_whitespace();
                match c {
                    '(' => self.push_token(TokenType::LParen),
                    ')' => self.push_token(TokenType::RParen),
                    '[' => self.push_token(TokenType::LBracket),
                    ']' => self.push_token(TokenType::RBracket),
                    '{' => self.push_token(TokenType::LBrace),
                    '}' => self.push_token(TokenType::RBrace),
                    ',' => self.push_token(TokenType::Comma),
                    '.' => self.push_token(TokenType::Period),
                    '?' => self.push_token(TokenType::Question),
                    ';' => self.push_token(TokenType::Semicolon),
                    ':' => self.push_token(TokenType::Colon),
                    '+' => self.push_token(TokenType::Plus),
                    '-' => self.push_token(TokenType::Minus),
                    '*' => self.push_token(TokenType::Times),
                    '%' => self.push_token(TokenType::Mod),
                    '&' => self.push_token(TokenType::And),
                    '|' => self.push_token(TokenType::Or),
                    '_' => self.push_token(TokenType::Underscore),
                    '/' => self.lex_slash(),
                    '!' => self.lex_exclamation(),
                    '<' => self.lex_langle(),
                    '>' => self.lex_rangle(),
                    '=' => self.lex_equal(),
                    '\"' => {
                        if let Err(e) = self.lex_string() {
                            self.push_token(TokenType::Error(format!("{:?}", e)));
                            break;
                        }
                    }
                    '\'' => {
                        if let Err(e) = self.lex_character() {
                            self.push_token(TokenType::Error(format!("{:?}", e)));
                            break;
                        }
                    }
                    other => {
                        if other.is_ascii_digit() {
                            if let Err(e) = self.lex_integer() {
                                self.push_token(TokenType::Error(format!("{:?}", e)));
                                break;
                            }
                        } else if other.is_ascii_alphabetic() {
                            self.lex_key_or_identifier();
                        } else {
                            self.push_token(TokenType::Error(format!(
                                "Unexpected character: {}",
                                other
                            )));
                            break;
                        }
                    }
                };
            }
            return Ok(Some(self.tokens));
        }
    }

    fn write_to_file(tokens: &mut VecDeque<Token>, output_file: PathBuf) -> Result<(), LexerError> {
        let output_text = tokens
            .drain(..)
            .map(|token| token.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        match fs::write(output_file.as_path(), output_text) {
            Ok(_) => Ok(()),
            Err(e) => {
                return Err(LexerError::IOError(
                    format!("{:?}", output_file.as_os_str()),
                    e,
                ));
            }
        }
    }

    pub fn lex_files<Tokens>(
        source_files: Vec<String>,
        path: PathBuf,
    ) -> Result<Option<Vec<(VecDeque<Token>, PathBuf)>>, Vec<LexerError>> {
        let mut err_vec = vec![];
        if let Err(e) = verify_files(&source_files) {
            err_vec.push(e);
            return Err(err_vec);
        };
        let mut token_queues: Vec<(VecDeque<Token>, PathBuf)> = vec![];
        let output_files = construct_paths(&source_files, path);
        for (input_file, output_file) in source_files.into_iter().zip(output_files.into_iter()) {
            let tokenizer = match Tokenizer::build(input_file) {
                Ok(t) => t,
                Err(e) => {
                    err_vec.push(e);
                    return Err(err_vec);
                }
            };
            match tokenizer.lex_file() {
                Ok(Some(q)) => token_queues.push((q, output_file)),
                Ok(None) => return Ok(None),
                //Only send file io errors upstream otherwise lexer errors should print error to lexed file
                //or maybe print to terminal if --paser or --compile selected?
                Err(e) => err_vec.push(e),
            };
        }
        if !err_vec.is_empty() {
            return Err(err_vec);
        }
        if true {
            for (mut token_stream, output_file) in token_queues {
                match write_to_file(&mut token_stream, output_file) {
                    Ok(_) => (),
                    Err(e) => {
                        err_vec.push(e);
                        return Err(err_vec);
                    }
                };
            }
            return Ok(None);
        }
        Ok(Some(token_queues))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_pass() {}
}
