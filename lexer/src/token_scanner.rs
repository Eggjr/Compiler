use crate::token::Token;
use crate::token_type::TokenType;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
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

    /// Creates token with TokenType `ttype` at current line and column
    fn create_token(&self, ttype: TokenType) -> Token {
        Token::new(self.line, self.column, ttype)
    }

    /// Creates token with TokenType `ttype` at current line and specified column `col`
    fn create_token_col(&self, col: usize, ttype: TokenType) -> Token {
        Token::new(self.line, col, ttype)
    }

    /// Consumes the next character from the `self.stream` and returns `Some(char)` if it was not empty and otherwise returns `None`
    /// Also handles incrementing the `self.line`, `self.column`, and `self.index`
    /// Convert any CRLF to just LF
    fn consume(&mut self) -> Option<char> {
        if let Some(c) = self.stream.next() {
            self.index += c.len_utf8();
            match c {
                '\n' => {
                    self.line += 1;
                    self.column = 0;
                    return Some('\n');
                }
                '\r' => {
                    let mut c_to_return = '\r';
                    if let Some(newline) = self.stream.peek()
                        && *newline == '\n'
                    {
                        self.stream.next();
                        self.index += 1; // '\n'.len_utf8() == 1
                        c_to_return = '\n';
                    }
                    self.line += 1;
                    self.column = 0;
                    return Some(c_to_return);
                }
                '\t' => {
                    self.column += 2;
                    return Some('\t');
                }
                ' ' => {
                    self.column += 1;
                    return Some(' ');
                }
                other => {
                    self.column += 1;
                    return Some(other);
                }
            };
        }
        None
    }

    /// Remove all whitespace from the front of `self.stream`
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

    /// Consumes the text on the line following a comment symbol: `//`
    fn consume_comment(&mut self) {
        while let Some(c) = self.consume() {
            if c == '\n' {
                return;
            }
        }
    }

    /// Lexes a forward slash `/` to determine if it is division of a comment
    /// returns Some(Token representation of division) (`/`), or returns None if comment (`//`)
    fn lex_slash(&mut self) -> Option<Token> {
        match self.stream.peek() {
            Some('/') => {
                self.consume_comment();
                None
            }
            _nonslash => Some(self.create_token(TokenType::Divide)),
        }
    }

    /// lexes `*` to determine if it is Times or High Multiplication
    /// returns Token representation of Times (`*`) or High Multiplication (`*>>`) or an error token if neither was matched
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

    /// Converts a hexadecimal escape enclosed in curly braces to its utf-8 char representation
    /// returns Ok(char) if it could convert the sequence or ERr(UnexpectedHexEscape) if the operation failed
    /// May fail if hex code is invalid utf-8, or curly braces are missing
    fn hex_to_char(&mut self) -> Result<char, UnexpectedHexEscape> {
        let start_col = self.column;
        let start_index = self.index + 1; //size of { is 1
        match self.consume() {
            Some('{') => (),
            _ => {
                return Err(UnexpectedHexEscape(format!(
                    "Expected {{ to begin Unicode Character hex value at {}:{}",
                    self.line, start_col
                )));
            }
        };
        let mut digits = 0;
        while let Some(c) = self.consume()
            && digits < 7
        {
            if c.is_ascii_hexdigit() {
                digits += 1;
            } else if c == '}' {
                if digits == 0 {
                    return Err(UnexpectedHexEscape(format!(
                        "Empty Hex Escape Given at {}:{}",
                        self.line, start_col
                    )));
                }
                let u32_rep =
                    u32::from_str_radix(&self.input_text[start_index..self.index - 1], 16)
                        .expect("Somehow Passed a Hexadecimal that Exceeded u32 limit");
                match char::from_u32(u32_rep) {
                    Some(c) => return Ok(c),
                    None => {
                        return Err(UnexpectedHexEscape(format!(
                            "Hexadecimal number {} could not be parsed to valid unicode",
                            &self.input_text[start_index..self.index - 1]
                        )));
                    }
                }
            } else {
                return Err(UnexpectedHexEscape(format!(
                    "Expected valid hexadecimal character but got: {}",
                    c
                )));
            }
        }
        Err(UnexpectedHexEscape(format!(
            "Expected }} to end Unicode Character value hex at {}:{}",
            self.line, start_col
        )))
    }

    /// Processes escape sequence and returns corresponding character or error message if failure occurred
    fn process_escape(&mut self) -> Result<char, String> {
        match self.consume() {
            Some('x') => match self.hex_to_char() {
                Ok(c) => Ok(c),
                Err(UnexpectedHexEscape(e)) => Err(e.to_string())
            },
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('\'') => Ok('\''),
            Some('\"') => Ok('\"'),
            Some('\\') => Ok('\\'),
            Some(c) => Err(format!("Unexpected Escape Sequence found: \\{}", c)),
            None => Err("Unfinished Escape Seqeunce. Expected literal to terminate with <escape>\' but got nothing".to_string())
        }
    }

    /// Lexes a character literal enclosed in single quotes (`''`)
    /// returns a token representation of that character literal or an error token if the character was invalid
    fn lex_character(&mut self) -> Token {
        let start_col = self.column;
        let val = match self.consume() {
            Some('\\') => match self.process_escape() {
                Ok(c) => c,
                Err(msg) => return self.create_token_col(start_col, TokenType::Error(msg)),
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

    /// lexes a string literal enclosed in double quotes (`""`)
    /// returns a token representation of the string literal or an error Token if there was a failure parsing the string
    fn lex_string(&mut self) -> Token {
        let mut contents = String::new();
        let start_col = self.column;
        while let Some(c) = self.consume() {
            match c {
                '\"' => {
                    return self.create_token_col(start_col, TokenType::String(contents));
                }
                '\\' => match self.process_escape() {
                    Ok(c) => contents.push(c),
                    Err(msg) => return self.create_token_col(start_col, TokenType::Error(msg)),
                },
                _ => contents.push(c),
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

    /// lexes the left angle bracket (`<`)
    /// returns a token representation of Less Than (`<`) or Less Than Or Equal (`<=`)
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

    /// lexes the right angle bracket (>)
    /// returns a token representation of Greater Than (>) or Greater Than Or Equal (>=)
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

    /// lexes the exclamation point (`!`)
    /// returns a Token representation of either unary boolean not operator (`!`) or the notequal operator (`!=`)
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

    /// lexes the equal sign (=)
    /// returns a token representation of the assignment operator (=) or the boolean equality operator (==)
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

    /// lexes an integer (sequence of number)
    /// returns an unsigned token representation of the lexed integer as `u64` that may or may not be valid `i32`
    /// may return an error token if it couldn't parse the int to a `u64` successfully
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

    /// Lexes a keyword or identifier with maximal munch
    /// returns a token representation of a keyword (`int`, `bool`, `while`, `use`, `return`, `length`, `true`, `false`, `if`, `else`)
    /// or an identifer (variable name, function name)
    fn lex_key_or_identifier(&mut self, ch: char) -> Token {
        let first = self.index - ch.len_utf8();
        let start_col = self.column;
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
        self.create_token_col(start_col, ttype)
    }

    /// Produces the next token from the input text
    /// returns 'Some(token)' if there is one or 'None' if no tokens are left
    /// Token produced may be an error token
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
                '\n' | '\t' | ' ' => return self.next_token(),
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
    use pretty_assertions::assert_eq;

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
        let cases = vec![
            (Some('1'), 1, 1, 1),
            (Some('\n'), 2, 0, 3),
            (Some('2'), 2, 1, 4),
            (Some('\n'), 3, 0, 5),
            (Some('3'), 3, 1, 6),
            (Some('\r'), 4, 0, 7),
            (Some('4'), 4, 1, 8),
            (Some('\t'), 4, 3, 9),
            (Some('5'), 4, 4, 10),
            (Some(' '), 4, 5, 11),
        ];
        for case in cases {
            assert_eq!(
                (
                    scanner.consume(),
                    scanner.line,
                    scanner.column,
                    scanner.index
                ),
                case
            );
        }

        assert!(scanner.consume().is_none());
    }

    #[test]
    fn test_remove_whitespace() {
        let mut scanner: TokenScanner<'_> = TokenScanner::new("\t     \nWhiteSpace\r\n    ");
        let chars = vec!['W', 'h', 'i', 't', 'e', 'S', 'p', 'a', 'c', 'e'];
        for i in 0..10 {
            scanner.remove_whitespace();
            assert_eq!(chars[i], scanner.consume().expect("None found"))
        }
        scanner.remove_whitespace();
        assert_eq!(None, scanner.consume());
    }

    #[test]
    fn test_consume_comment() {
        let mut scanner: TokenScanner<'_> = TokenScanner::new(
            "//This Comment Will Be Consumed\nnot_this//Same line Comment\n//Also not read",
        );
        let chars = vec!['n', 'o', 't', '_', 't', 'h', 'i', 's'];
        scanner.consume_comment();
        for i in 0..8 {
            assert_eq!(chars[i], scanner.consume().expect("None found"))
        }
        scanner.consume_comment();
        scanner.consume_comment();
        assert!(scanner.consume().is_none())
    }

    #[test]
    fn test_lex_slash() {
        let mut scanner = TokenScanner::new("1/2//COMMENT");
        let cases = vec![Some('1')];
        scanner.consume();
        scanner.consume();
        assert_eq!(
            scanner.lex_slash().unwrap(),
            Token::new(1, 2, TokenType::Divide)
        );
        scanner.consume();
        assert_eq!(scanner.lex_slash(), None);
        assert_eq!(scanner.consume(), None);
    }

    #[test]
    fn test_lex_star() {
        let mut cases = vec![
            ("1*2", 2, Token::new(1, 2, TokenType::Times)),
            (
                "2147483668*>>2147483668",
                11,
                Token::new(1, 11, TokenType::HighMultiplication),
            ),
            (
                "*>a",
                1,
                Token::new(
                    1,
                    1,
                    TokenType::Error(format!(
                        "Expected > to finish High Multiplication token but received {}",
                        'a'
                    )),
                ),
            ),
            (
                "*>",
                1,
                Token::new(
                    1,
                    1,
                    TokenType::Error(
                        "Expected > to finish High Mulitplication Token but found nothing"
                            .to_string(),
                    ),
                ),
            ),
        ];
        for (input, consume, output) in cases {
            let mut scanner = TokenScanner::new(input);
            for i in 0..consume {
                scanner.consume();
            }
            assert_eq!(scanner.lex_star(), output)
        }
    }

    #[test]
    fn test_hex_char() {
        let non_hex_chars = vec![
            '`', '~', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '+', '=', '[',
            '{', ']', '\\', '|', '\'', '\"', ':', ';', '<', ',', '>', '.', '/', '?', 'g', 'h', 'i',
            'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'R',
            'X', 'Y', 'Z',
        ];
        let mut cases = vec![
            ("{10FFFF}".to_string(), Ok('\u{10FFFF}')),
            (
                "ABCDEF}".to_string(),
                Err(UnexpectedHexEscape(format!(
                    "Expected {{ to begin Unicode Character hex value at {}:{}",
                    1, 0
                ))),
            ),
            (
                "{AAA AAA}".to_string(),
                Err(UnexpectedHexEscape(
                    "Expected valid hexadecimal character but got:  ".to_string(),
                )),
            ),
            (
                "{11FFFF}".to_string(),
                Err(UnexpectedHexEscape(format!(
                    "Hexadecimal number {} could not be parsed to valid unicode",
                    "11FFFF"
                ))),
            ),
            (
                "{124456".to_string(),
                Err(UnexpectedHexEscape(format!(
                    "Expected }} to end Unicode Character value hex at {}:{}",
                    1, 0
                ))),
            ),
            (
                "{}".to_string(),
                Err(UnexpectedHexEscape(format!(
                    "Empty Hex Escape Given at {}:{}",
                    1, 0
                ))),
            ),
            (
                "{1111111}".to_string(),
                Err(UnexpectedHexEscape(format!(
                    "Expected }} to end Unicode Character value hex at {}:{}",
                    1, 0
                ))),
            ),
        ];
        for c in non_hex_chars {
            cases.push((
                format!("{{{}}}", &c),
                Err(UnexpectedHexEscape(format!(
                    "Expected valid hexadecimal character but got: {}",
                    c
                ))),
            ))
        }
        for (input, output) in cases {
            let mut scanner = TokenScanner::new(input.as_str());
            assert_eq!(scanner.hex_to_char(), output);
        }
    }

    #[test]
    fn test_lex_char() {
        let cases = vec![
            (r#"'\n'"#, 1 ,Token::new(1, 1, TokenType::Character('\n'))),
            (r#"'\t'"#, 1, 
                Token::new(1, 1, TokenType::Character('\t'))
            ),
            (r#"'\r'"#, 1, 
                Token::new(1, 1, TokenType::Character('\r'))
            ),
            (r#"'\''"#, 1, 
                Token::new(1, 1, TokenType::Character('\''))
            ),
            (r#"'\"'"#, 1, 
                Token::new(1, 1, TokenType::Character('\"'))
            ),
            (r#"'\\'"#, 1, 
                Token::new(1, 1, TokenType::Character('\\'))
            ),
            (r#"'\x{64}'"#, 1, 
                Token::new(1, 1, TokenType::Character('d'))
            ),
            ("\'\'", 1, Token::new(1, 1, TokenType::Error("No Character Given".to_string()))),
            ("\'\\b\'", 1,
                Token::new(
                    1,
                    1,
                    TokenType::Error("Unexpected Escape Sequence found: \\b".to_string())
                )),
            ("\'\\x{FFFFFF}\'", 1,
                Token::new(
                    1,
                    1,
                    TokenType::Error(
                        "Hexadecimal number FFFFFF could not be parsed to valid unicode".to_string()
                    )
                )),
            ("\'\\", 1,
                Token::new(
                    1,
                    1,
                    TokenType::Error(String::from(
                        "Unfinished Escape Seqeunce. Expected literal to terminate with <escape>\' but got nothing",
                    ))
                )
            ),
            ("\'aa\'", 1,
                Token::new(
                    1,
                    1,
                    TokenType::Error(format!(
                        "Expected character literal to be closed with \' but got {}",
                        'a'
                    ))
                )
            ),
            ("\'a", 1, 
                Token::new(1, 1, TokenType::Error("Unfinished Escape Seqeunce. Expected literal to terminate with <escape>\' but got nothing".to_string()))),
            ("\'", 1, 
                Token::new(
                    1,
                    1,
                    TokenType::Error(
                        "Expected character literal to be closed with \' but got nothing".to_string()
                    )
                )
            ),
            ("\'z\'", 1,
                Token::new(1, 1, TokenType::Character('z'))
            ),
        ];
        for (input, consume, output) in cases {
            let mut scanner = TokenScanner::new(input);
            for i in 0..consume {
                scanner.consume();
            }
            assert_eq!(scanner.lex_character(), output);
        }
    }

    #[test]
    fn test_lex_string() {
        let mut scanner = TokenScanner::new("\"Hello Worl\\x{64}!\"");
        let mut err_scanner = TokenScanner::new("\"Hello");
        let mut err_scanner2 = TokenScanner::new("\"");
    }

    #[test]
    fn test_next_token() {
        let mut scanner: TokenScanner<'_> = TokenScanner::new(&INPUT_TEXT);
        let str_output = "1:1 id i\n\
            1:3 :\n\
            1:5 int\n\
            1:9 =\n\
            1:11 integer 0\n\
            2:1 id z\n\
            2:2 :\n\
            2:3 int\n\
            2:6 =\n\
            2:7 integer 1\n\
            2:8 +\n\
            2:9 integer 2\n\
            2:10 +\n\
            2:11 integer 3\n\
            3:1 id s\n\
            3:2 :\n\
            3:4 int\n\
            3:7 [\n\
            3:8 ]\n\
            3:10 =\n\
            3:12 string Hello\n\
            4:1 id b\n\
            4:2 :\n\
            4:3 bool\n\
            4:7 ,\n\
            4:9 id i\n\
            4:10 :\n\
            4:11 int\n\
            4:15 =\n\
            4:17 id f\n\
            4:18 (\n\
            4:19 id x\n\
            4:20 )\
        ";
        let mut tokens = vec![];
        while let Some(t) = scanner.next_token() {
            tokens.push(t);
        }
        assert_eq!(
            tokens
                .into_iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
            str_output
        )
    }
}
