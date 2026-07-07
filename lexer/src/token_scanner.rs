use crate::token::Token;
use crate::token_scanner::HexLexError::UnexpectedHexEscape;
use crate::token_type::TokenType;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub enum HexLexError {
    UnexpectedHexEscape(String),
}

#[derive(Debug, Clone)]
pub struct TokenScanner<'a> {
    stream: Peekable<Chars<'a>>,
    input_text: &'a str,
    line: usize,
    column: usize,
    index: usize,
}

impl<'a> TokenScanner<'a> {
    pub fn build(input_text: &'a str) -> TokenScanner<'a> {
        TokenScanner {
            line: 1,
            column: 1,
            index: 0,
            stream: input_text.chars().peekable(),
            input_text,
        }
    }

    fn create_token(&self, ttype: TokenType) -> Token {
        Token::new(self.line, self.column, ttype)
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
                            self.index += 1 // '\n'.len_utf8() == 1
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

    fn remove_whitespace(&mut self) {
        while let Some(c) = self.stream.peek() {
            match c {
                '\n' | '\t' | ' ' | '\r' => {
                    let _ = self.consume();
                }
                _nonwhitespace => return,
            }
        }
    }

    fn consume_comment(&mut self) {
        while let Some(c) = self.consume() {
            if c == '\n' {
                return;
            }
        }
    }

    fn lex_slash(&mut self) -> Option<Token> {
        self.consume();
        match self.stream.peek() {
            Some('/') => {
                self.consume_comment();
                None
            }
            _nonslash => Some(self.create_token(TokenType::Divide)),
        }
    }

    fn hex_to_char(&mut self) -> Result<char, HexLexError> {
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
        };
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

    fn lex_character(&mut self) -> Token {
        let val = match self.consume() {
            Some('\\') => match self.consume() {
                Some('x') => match self.hex_to_char() {
                    Ok(c) => c,
                    Err(HexLexError::UnexpectedHexEscape(e)) => return self.create_token(TokenType::Error(format!("{:?}", e))),
                },
                Some('n') => '\n',
                Some('r') => '\r',
                Some('t') => '\t',
                Some('\'') => '\'',
                Some('\"') => '\"',
                Some('\\') => '\\',
                Some(c) => {
                    return self.create_token(TokenType::Error(format!(
                        "Unexpected Escape Sequence found: \\{}",
                        c
                    )));
                }
                None => {
                    return self.create_token(TokenType::Error(String::from(
                        "Unfinished Escape Seqeunce. Expected literal to terminate with <escape>\' but got nothing",
                    )));
                }
            },
            Some('\'') => {
                return self.create_token(TokenType::Error(String::from("No Character Given")));
            }
            Some(c) => c,
            None => {
                return self.create_token(TokenType::Error(String::from(
                    "Expected character literal to be closed with \' but got nothing",
                )));
            }
        };
        if let Some(c) = self.consume() {
            match c {
                '\'' => {
                    return self.create_token(TokenType::Character(val));
                }
                c => {
                    return self.create_token(TokenType::Error(format!(
                        "Expected character literal to be closed with \' but got {}",
                        c
                    )));
                }
            }
        }
        return self.create_token(TokenType::Error(String::from(
            "Unfinished Escape Seqeunce. Expected literal to terminate with <escape>\' but got nothing",
        )));
    }

    fn lex_string(&mut self) -> Token {
        let mut contents = String::new();
        let start_col = self.column - 1;
        while let Some(c) = self.consume() {
            match c {
                '\"' => {
                    return Token::new(self.line, start_col, TokenType::String(contents));
                }
                '\\' => match self.stream.peek() {
                    Some('x') => contents.push(match self.hex_to_char() {
                        Ok(c) => c,
                        Err(HexLexError::UnexpectedHexEscape(e)) => {
                            return self.create_token(TokenType::Error(format!("{:?}", e)));
                        }
                    }),
                    _ => contents.push(c), //if not a hex escape add the backslash
                },
                _ => {
                    contents.push(c);
                }
            }
        }
        return self.create_token(TokenType::Error(format!(
            "Expected \" to close String at {}:{}",
            self.line,
            self.column - 1
        )));
    }

    fn lex_langle(&mut self) -> Token {
        let token: Token;
        match self.stream.peek() {
            Some('=') => {
                token = self.create_token(TokenType::LE);
                self.consume();
            }
            _ => token = self.create_token(TokenType::LAngle),
        }
        token
    }

    fn lex_rangle(&mut self) -> Token {
        let token: Token;
        match self.stream.peek() {
            Some('=') => {
                token = self.create_token(TokenType::GE);
                self.consume();
            }
            _ => token = self.create_token(TokenType::RAngle),
        }
        token
    }

    fn lex_exclamation(&mut self) -> Token {
        let token: Token;
        match self.stream.peek() {
            Some('=') => {
                token = self.create_token(TokenType::NE);
                self.consume();
            }
            _ => token = self.create_token(TokenType::Exclamation),
        }
        token
    }

    fn lex_equal(&mut self) -> Token {
        let token: Token;
        match self.stream.peek() {
            Some('=') => {
                token = self.create_token(TokenType::EQ);
                self.consume();
            }
            _ => token = self.create_token(TokenType::Assign),
        }
        token
    }

    fn lex_integer(&mut self) -> Token {
        let first = self.index - 1;
        while let Some(c) = self.stream.peek() {
            if !c.is_ascii_digit() {
                break;
            } else {
                self.consume();
            }
        }
        //Handle i64 bounds in the parser
        return self.create_token(TokenType::Integer(
            match self.input_text[first..self.index].parse::<u64>() {
                Ok(val) => val,
                Err(e) => {
                    return self
                        .create_token(TokenType::Error(format!("Trouble Parsing int: {:?}", e)));
                }
            },
        ));
    }

    fn lex_key_or_identifier(&mut self) -> Token {
        let first = self.index - 1;
        while let Some(c) = self.stream.peek() {
            if !(c.is_ascii_alphanumeric() || *c == '_' || *c == '\'') {
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
        return self.create_token(ttype);
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.remove_whitespace();
        if let Some(c) = self.consume() {
            let token = match c {
                '(' => self.create_token(TokenType::LParen),
                ')' => self.create_token(TokenType::RParen),
                '[' => self.create_token(TokenType::LBracket),
                ']' => self.create_token(TokenType::RBracket),
                '{' => self.create_token(TokenType::LBrace),
                '}' => self.create_token(TokenType::RBrace),
                ',' => self.create_token(TokenType::Comma),
                '.' => self.create_token(TokenType::Period),
                '?' => self.create_token(TokenType::Question),
                ';' => self.create_token(TokenType::Semicolon),
                ':' => self.create_token(TokenType::Colon),
                '+' => self.create_token(TokenType::Plus),
                '-' => self.create_token(TokenType::Minus),
                '*' => self.create_token(TokenType::Times),
                '%' => self.create_token(TokenType::Mod),
                '&' => self.create_token(TokenType::And),
                '|' => self.create_token(TokenType::Or),
                '_' => self.create_token(TokenType::Underscore),
                '!' => self.lex_exclamation(),
                '<' => self.lex_langle(),
                '>' => self.lex_rangle(),
                '=' => self.lex_equal(),
                '/' => match self.lex_slash() {
                    Some(t) => t,
                    None => return self.next_token(),
                },
                '\"' => self.lex_string(),
                '\'' => self.lex_character(),
                other => {
                    if other.is_ascii_digit() {
                        self.lex_integer()
                    } else if other.is_ascii_alphabetic() {
                        self.lex_key_or_identifier()
                    } else {
                        self.create_token(TokenType::Error(format!(
                            "Unexpected character: {}",
                            other
                        )))
                    }
                }
            };
            return Some(token);
        }
        None
    }
}
