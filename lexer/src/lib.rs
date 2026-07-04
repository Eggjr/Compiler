pub mod token_type;
pub mod token;

pub mod lexer{
    use std::fs;
    use std::fs::File;
    use std::io::SeekFrom::Current;
    use std::path::PathBuf;
    use std::str::Chars;
    use std::iter::Peekable;
    use std::collections::VecDeque;
    use crate::lexer::LexerError::{IOError, IntegerTooLarge, InvalidFilesError, UnclosedCharacter, UnexpectedCharacterConstant, UnexpectedHexEscape, WorldBroken};
    use crate::token_type::token_type::TokenType;
    use crate::token::token::Token;

    #[derive(Debug)]
    pub enum LexerError{
        InvalidFilesError(Vec<String>),
        IOError(String, std::io::Error),
        WorldBroken(String),
        UnexpectedCharacterConstant(String),
        NoMoreTokens(String),
        UnclosedString(String),
        UnexpectedHexEscape(String),
        UnexpectedEscapeSequence(String),
        UnclosedCharacter(String),
        IntegerTooLarge(String),
    }

    fn verify_files(source_files : &[String]) -> Result<(), LexerError>{
        let invalids : Vec<String> = source_files
        .iter()
        .filter(|file| !file.ends_with(".eta") && !file.ends_with(".eti"))
        .cloned()
        .collect();
        if !invalids.is_empty(){
            return Err(InvalidFilesError(invalids));
        }
        return Ok(());
    }

    fn construct_paths(source_files : &[String], path : PathBuf) -> Vec<PathBuf>{
        return source_files
            .iter()
            .map(|file_name| (path.join(file_name)).with_extension("lexed"))
            .collect();
    }

    #[derive(Debug, Clone)]
    struct Tokenizer<'a>{
        line : usize,
        column : usize,
        stream : Peekable<Chars<'a>>,
        input_file : &'a str,
        tokens : VecDeque<Token>,
        output_file : PathBuf,
        index : usize,
        input_text : &'a str,
    }
    
    impl<'a> Tokenizer<'a>{
        fn build(input_file : &str, output_file : PathBuf) -> Result<Tokenizer, LexerError>{
            let file_contents = match fs::read_to_string(source_file){
                Ok(file_contents) => file_contents,
                Err(e) => {return Err(IOError(input_file.to_owned(), e));}
            };
            let mut stream= file_contents.chars().peekable();
            Ok(Tokenizer{line:1, column:1, stream, input_file, tokens:VecDeque::new(), output_file, index:0, input_text:&file_contents})
        }


        fn consume(&mut self) -> Option<char>{
            if let Some(c) = self.stream.next(){
                self.index += 1;
                match c{
                    '\n' => {
                        self.line += 1;
                        self.column = 1;
                        return None
                    },
                    '\r' =>{
                        if let Some(newline) = self.stream.peek(){
                            if newline == '\n'{
                                self.stream.next();
                                self.index += 1
                            }
                        }
                        self.line += 1;
                        self.column = 1;
                        return None
                    },
                    other => {
                        self.column += 1;
                        return Some(other)
                    }
                };
            }
            None
        }

        fn remove_whitespace(&mut self) -> (){
            while let Some(c) = self.stream.peek(){
                match c{
                    '\n' | '\t' | ' ' | '\r' => self.consume(),
                    other => return ()
                }
            }
            return ()
        }

        fn consume_comment(&mut self) -> (){
            while let Some(c) = self.consume(){
                if c == '\n'{
                    break
                }
            }
        }

        fn lex_slash(&mut self) -> (){
            self.consume();
            match self.stream.peek(){
                Some('/') => self.consume_comment(),
                _ => self.tokens.push_back(self.create_token(TokenType::Divide))
            }
            ()
        }

        fn hex_to_char(&mut self) -> Result<char, LexerError>{
            let mut res : String = "";
            match self.consume(){
                Some('{') => (),
                _ => return Err(UnexpectedHexEscape(format!("Expected {{ to begin Unicode Character hex value hex at {}:{}", self.line-1, self.column)))
            }
            let mut digits = 0;
            while let Some(c) = self.consume() && digits < 6{
                if c.is_ascii_hexdigit(){
                    res.push(c);
                    digits += 1;
                }
                else if c == '}'{
                    let u32_rep = match u32::from_str_radix(res.as_str(), 16){
                        Ok(u) => u,
                        Err(e) => return Err(UnexpectedHexEscape(format!("{}", e)))
                    };
                    match char::from_u32(u32_rep){
                            Some(c) => return Ok(c),
                            None => return Err(UnexpectedHexEscape(format!("Hexadecimal number {} could not be parsed to valid unicode", res)))
                    }
                }
                else{
                    return Err(UnexpectedHexEscape(format!("Expected valid hexadecimal character but got {}", c)));
                }
            }
            Err(UnexpectedHexEscape(format!("Expected }} to end Unicode Character value hex at {}:{}", self.line-1, self.column)))
        }

        fn lex_character(&mut self) -> Result<(), LexerError>{
            let val = match self.consume(){
                Some('\\') => match self.stream.consume(){
                    Some('x') => self.hex_to_char()?,
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('\'') => '\'',
                    Some('\"') => '\"',
                    Some('\\') => '\\',
                    Some(c) => return Err(UnexpectedEscapeSequence(format!("Unexpected Escape Sequence found: \\{}", c))),
                    None => return Err(UnexpectedHexEscape("Character did not terminate. Expected <char>\' but got nothing"))
                },
                Some('\'')=> return Err(UnexpectedCharacterConstant(format!("No Character Given"))),
                Some(c) => c,
                None => return Err(UnclosedCharacter(format!("Expected character literal to be closed with \'")))
            };
            if let Some(c) = self.consume(){
                match c {
                    '\'' => {
                        self.push_token(TokenType::Character(val));
                        return Ok(());
                    }
                    c => return Err(UnclosedCharacter(format!("Expected character literal to be closed with \' but got }|", c)))
                }
            }
        }

        fn lex_string(&mut self) -> Result<(), LexerError>{
            let mut contents = String::new();
            let start_col = self.column - 1;
            while let Some(c) = self.consume(){
                match c{
                    '\"' => {
                        self.tokens.push_back(Token::new(self.line, start_col, TokenType::String(contents)));
                        return Ok(())
                    },
                    '\\' => match self.stream.peek(){
                        Some('x') => contents.push(self.hex_to_char()?),
                        _ => ()
                    }, 
                    _ => {
                        contents.push(c);
                    }
                }
            } 
            return Err(UnclosedString(format!("Expected \" to close String at {}:{}", self.line, self.column)))
        }

        fn lex_langle(&mut self){
            match self.stream.peek(){
                Some('=') => {
                    self.push_token(TokenType::LE);
                    self.consume();
                }
                _ => self.push_token(TokenType::LAngle)
           }
        }

        fn lex_rangle(&mut self){
            match self.stream.peek(){
                Some('=') => {
                    self.push_token(TokenType::GE);
                    self.consume();
                }
                _ => self.push_token(TokenType::RAngle)
           }
        }

        fn lex_exclamation(&mut self){
            match self.stream.peek(){
                Some('=') => {
                    self.push_token(TokenType::NE);
                    self.consume();
                },
                _ => self.push_token(TokenType::Exclamation)
            }
        }

        fn lex_equal(&mut self){
            match self.stream.peek(){
                Some('=') => {
                    self.push_token(TokenType::EQ);
                    self.consume();
                },
                _ => {
                    self.push_token(TokenType::Assign)
                }
            }
        }

        fn lex_integer(&mut self){
            let first  = self.index - 1;
            while let Some(c) = self.stream.peek(){
                if !c.is_ascii_digit(){
                    break;
                }else{
                    self.consume();
                }
            }
            //Handle i64 bounds in the parser
            self.push_token(TokenType::Integer(
                match self.input_text[first..self.index].parse::<u64>(){
                    Ok(val) => val,
                    Err(e) => return Err(IntegerTooLarge(format!("Trouble Parsing int: {}", e)))
                }
            ));
        }

        fn lex_key_or_identifier(&mut self){
            let first = self.index - 1;
            while let Some(c) = self.stream.peek(){
                if !(c.is_ascii_alphanumeric() || c == '_' || c == '\''){
                    break;
                }
                else{
                    self.consume();
                }
            };
            let ttype = match self.input_text[first..self.index]{
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
                identifier => TokenType::Identifier(identifier),
            };
            self.push_token(ttype);
        }

        
        fn push_token(&mut self, token_type:TokenType) -> (){
            self.tokens.push_back(Token::new(self.line, self.column-1, token_type));
        }


        //FIX THIS: We want to print Error tokens to file not output them to main unless they are io errors
        fn lex_file(&mut self) -> Result<VecDeque<Token>, LexerError>{
            while let Some(c) = self.consume(){
                self.remove_whitespace();
                match c{
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
                    '\"' => match self.lex_string(){
                        Err(e) => {
                            self.push(TokenType::Error(e)); 
                            Err(format!("{:?}", e));
                            break;
                        },
                        _ => (),
                    },
                    '\'' => match self.lex_character(){
                        Err(e) => {
                            self.push(TokenType::Error(e));
                            Err(format!("{:?}", e));
                            break;
                        }
                        _ => (),
                    },
                    other => {
                        if other.is_ascii_digit(){
                            self.lex_integer();
                        }
                        else if other.is_ascii_alphabetic(){
                            self.lex_key_or_identifier();
                        }
                        else{
                            self.push_token(TokenType::Error(format!("Unexpected character: {}", other)));
                            break;
                        }
                    }
                };
            }
            let output_text = Vec::from(self.tokens.clone())
                .iter()
                .map(|token| token.to_string())
                .collect::<Vec<String>>()
                .join("\n");
            //Future: Add ability to output to file or send to parser
            fs::write(self.output_file.as_path(), output_text);
            return Ok(self.tokens)
        }
    }

    pub fn lex_files<Tokens>(source_files : &[String], path : PathBuf) -> Result<Vec<(VecDeque<Token>, PathBuf)>, LexerError>{
        verify_files(source_files)?;
        let output_files = construct_paths(source_files, path);
        let mut token_queues : Vec<(VecDeque<Token>, PathBuf)> = vec![];
        for (input_file, output_file) in source_files.iter().zip(output_files){
            let mut tokenizer = Tokenizer::build(input_file, output_file.clone())?;
            match tokenizer.lex_file(){
                Ok(q) => token_queues.push((q, output_file)),
                //Only send file io errors upstream otherwise lexer errors should print error to lexed file
                //or maybe print to terminal if --paser or --compile selected?
                Err(e) => return Err(e)
            };
        }  
        return Ok(token_queues)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_pass(){
        
    }

}
