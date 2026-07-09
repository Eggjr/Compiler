use crate::token::Token;
use crate::token_type::TokenType;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
struct UnexpectedHexEscape(String);

#[derive(Debug, Clone)]
pub(crate) struct TokenScanner<'a> {
    stream: Peekable<Chars<'a>>,
    input_text: &'a str,
    line: usize,
    column: usize,
    index: usize,
}

impl<'a> TokenScanner<'a> {
    pub(crate) fn new(input_text: &'a str) -> TokenScanner<'a> {
        TokenScanner {
            line: 1,
            column: 0,
            index: 0,
            stream: input_text.chars().peekable(),
            input_text,
        }
    }

    fn create_token(&self, ttype: TokenType) -> Token {
        Token::new(self.line, self.column, ttype)
    }

    fn create_token_col(&self, column: usize, ttype: TokenType) -> Token {
        Token::new(self.line, column, ttype)
    }

    fn consume(&mut self) -> Option<char> {
        if let Some(c) = self.stream.next() {
            self.index += c.len_utf8();
            match c {
                '\n' => {
                    self.line += 1;
                    self.column = 0;
                    return None;
                }
                '\r' => {
                    if let Some(newline) = self.stream.peek()
                        && *newline == '\n'
                    {
                        self.stream.next();
                        self.index += 1 // '\n'.len_utf8() == 1
                    }
                    self.line += 1;
                    self.column = 0;
                    return None;
                }
                '\t' => {
                    self.column += 2;
                    return None;
                }
                ' ' => {
                    self.column += 1;
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
        match self.stream.peek() {
            Some('/') => {
                self.consume_comment();
                None
            }
            _nonslash => Some(self.create_token(TokenType::Divide)),
        }
    }

    fn lex_star(&mut self) -> Token {
        let start_col = self.column;
        if let Some('>') = self.stream.peek() {
            let _ = self.consume();
            match self.consume() {
                Some('>') => {
                    return self.create_token_col(start_col, TokenType::HighMultiplication);
                }
                Some(non_rangle) => {
                    return self.create_token_col(
                        start_col,
                        TokenType::Error(format!(
                            "Expected > to finish High Multiplication token but received {}",
                            non_rangle
                        )),
                    );
                }
                None => {
                    return self.create_token_col(
                        start_col,
                        TokenType::Error(
                            "Expected > to finish High Mulitplication Token but found nothing"
                                .to_string(),
                        ),
                    );
                }
            }
        }
        self.create_token(TokenType::Times)
    }

    fn hex_to_char(&mut self) -> Result<char, UnexpectedHexEscape> {
        let mut res: String = String::from("");
        let start_col = self.column;
        match self.consume() {
            Some('{') => (),
            _ => {
                return Err(UnexpectedHexEscape(format!(
                    "Expected {{ to begin Unicode Character hex value hex at {}:{}",
                    self.line, start_col
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
            self.line, start_col
        )))
    }

    fn lex_character(&mut self) -> Token {
        let start_col = self.column;
        let val = match self.consume() {
            Some('\\') => match self.consume() {
                Some('x') => match self.hex_to_char() {
                    Ok(c) => c,
                    Err(UnexpectedHexEscape(e)) => {
                        return self.create_token(TokenType::Error(format!("{:?}", e)));
                    }
                },
                Some('n') => '\n',
                Some('r') => '\r',
                Some('t') => '\t',
                Some('\'') => '\'',
                Some('\"') => '\"',
                Some('\\') => '\\',
                Some(c) => {
                    return self.create_token_col(
                        start_col,
                        TokenType::Error(format!("Unexpected Escape Sequence found: \\{}", c)),
                    );
                }
                None => {
                    return self.create_token_col(start_col, TokenType::Error(String::from(
                        "Unfinished Escape Seqeunce. Expected literal to terminate with <escape>\' but got nothing",
                    )));
                }
            },
            Some('\'') => {
                return self.create_token_col(
                    start_col,
                    TokenType::Error(String::from("No Character Given")),
                );
            }
            Some(c) => c,
            None => {
                return self.create_token_col(
                    start_col,
                    TokenType::Error(String::from(
                        "Expected character literal to be closed with \' but got nothing",
                    )),
                );
            }
        };
        if let Some(c) = self.consume() {
            match c {
                '\'' => {
                    return self.create_token_col(start_col, TokenType::Character(val));
                }
                c => {
                    return self.create_token_col(
                        start_col,
                        TokenType::Error(format!(
                            "Expected character literal to be closed with \' but got {}",
                            c
                        )),
                    );
                }
            }
        }
        self.create_token_col(start_col, TokenType::Error(String::from(
            "Unfinished Escape Seqeunce. Expected literal to terminate with <escape>\' but got nothing",
        )))
    }

    fn lex_string(&mut self) -> Token {
        let mut contents = String::new();
        let start_col = self.column;
        while let Some(c) = self.consume() {
            match c {
                '\"' => {
                    return self.create_token_col(start_col, TokenType::String(contents));
                }
                '\\' => match self.stream.peek() {
                    Some('x') => contents.push(match self.hex_to_char() {
                        Ok(c) => c,
                        Err(UnexpectedHexEscape(e)) => {
                            return self
                                .create_token_col(start_col, TokenType::Error(format!("{:?}", e)));
                        }
                    }),
                    _ => contents.push(c), //if not a hex escape add the backslash
                },
                _ => {
                    contents.push(c);
                }
            }
        }
        self.create_token_col(
            start_col,
            TokenType::Error(format!(
                "Expected \" to close String at {}:{}",
                self.line, start_col
            )),
        )
    }

    fn lex_langle(&mut self) -> Token {
        let token: Token;
        match self.stream.peek() {
            Some('=') => {
                token = self.create_token_col(self.column - 1, TokenType::LE);
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
                token = self.create_token_col(self.column - 1, TokenType::GE);
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
                token = self.create_token_col(self.column - 1, TokenType::NE);
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
                token = self.create_token_col(self.column - 1, TokenType::EQ);
                self.consume();
            }
            _ => token = self.create_token(TokenType::Assign),
        }
        token
    }

    fn lex_integer(&mut self, ch: char) -> Token {
        let first = self.index - ch.len_utf8();
        let start_col = self.column;
        while let Some(c) = self.stream.peek() {
            if !c.is_ascii_digit() {
                break;
            } else {
                self.consume();
            }
        }
        //Handle i64 bounds in the parser
        self.create_token_col(
            start_col,
            TokenType::Integer(match self.input_text[first..self.index].parse::<u64>() {
                Ok(val) => val,
                Err(e) => {
                    return self.create_token_col(
                        start_col,
                        TokenType::Error(format!("Trouble Parsing int: {:?}", e)),
                    );
                }
            }),
        )
    }

    fn lex_key_or_identifier(&mut self, ch: char) -> Token {
        let first = self.index - ch.len_utf8();
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
        Token::new(self.line, first + 1, ttype)
    }

    pub(crate) fn next_token(&mut self) -> Option<Token> {
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
                '%' => self.create_token(TokenType::Mod),
                '&' => self.create_token(TokenType::And),
                '|' => self.create_token(TokenType::Or),
                '_' => self.create_token(TokenType::Underscore),
                '*' => self.lex_star(),
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
                        self.lex_integer(other)
                    } else if other.is_ascii_alphabetic() {
                        self.lex_key_or_identifier(other)
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

#[cfg(test)]
mod tests {
    use super::*;
    const WHITESPACE: &str = "1\r\n2\n3\r4\t5 ";
    const INPUT_TEXT: &str =
        "i : int = 0\r\nz:int=1+2+3\ns: int[] = \"Hello\"\nb:bool, i:int = f(x)";

    #[test]
    fn test_consume_whitespace() {
        let mut scanner: TokenScanner<'_> = TokenScanner::new(WHITESPACE);
        assert!(
            scanner.line == 1 && scanner.column == 0 && scanner.index == 0,
            "Debug Info:{scanner:?}"
        );
        assert!(
            scanner.consume() == Some('1')
                && scanner.line == 1
                && scanner.column == 1
                && scanner.index == 1,
            "Debug Info:{scanner:?}"
        );
        assert!(
            scanner.consume() == None
                && scanner.line == 2
                && scanner.column == 0
                && scanner.index == 3,
            "Debug Info:{scanner:?}"
        );
        assert!(
            scanner.consume() == Some('2')
                && scanner.line == 2
                && scanner.column == 1
                && scanner.index == 4,
            "Debug Info:{scanner:?}"
        );
        assert!(
            scanner.consume() == None
                && scanner.line == 3
                && scanner.column == 0
                && scanner.index == 5,
            "Debug Info:{scanner:?}"
        );
        assert!(
            scanner.consume() == Some('3')
                && scanner.line == 3
                && scanner.column == 1
                && scanner.index == 6,
            "Debug Info:{scanner:?}"
        );
        assert!(
            scanner.consume() == None
                && scanner.line == 4
                && scanner.column == 0
                && scanner.index == 7,
            "Debug Info:{scanner:?}"
        );
        assert!(
            scanner.consume() == Some('4')
                && scanner.line == 4
                && scanner.column == 1
                && scanner.index == 8,
            "Debug Info:{scanner:?}"
        );
        assert!(
            scanner.consume() == None
                && scanner.line == 4
                && scanner.column == 3
                && scanner.index == 9,
            "Debug Info:{scanner:?}"
        );
        assert!(
            scanner.consume() == Some('5')
                && scanner.line == 4
                && scanner.column == 4
                && scanner.index == 10,
            "Debug Info:{scanner:?}"
        );
        assert!(
            scanner.consume() == None
                && scanner.line == 4
                && scanner.column == 5
                && scanner.index == 11,
            "Debug Info: {scanner:?}"
        );
    }

    #[test]
    fn test_consume() {
        let mut _scanner: TokenScanner<'_> = TokenScanner::new(&INPUT_TEXT);
    }
}
