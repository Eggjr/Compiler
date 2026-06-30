
pub mod lexer{
    use std::fs::File;
    use std::io::{BufReader, BufRead};
    use std::path::PathBuf;
    use crate::lexer::LexerError::{IOError, InvalidFilesError};

    #[derive(Debug)]
    pub enum LexerError{
        InvalidFilesError(Vec<String>),
        IOError(String, std::io::Error),
    }

    #[derive(Debug, Clone)]
    enum TokenType{
        Identifier(String),
        Integer(i64),
        Character(char),
        String(String),
        LParen, // (
        RParen, // )
        LBracket, // [
        RBracket, // ]
        LBrace, // {
        RBrace, // }
        LAngle, // <
        RAngle, // >
        Semicolon, // ;
        Colon, // :
        Plus, // +
        Times, // *
        HighMultiplication, // *>>
        Minus, // -
        Divide, // /
        Mod, // %
        EQ, // ==
        NE, // !=
        LE, // <=
        GE, // >=
        Assign, // =
        And, // &
        Or, // |
        Exclamation, // !
        Period, // .
        Comma, // ,
        Question, // ?
        Underscore, // _
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
        EOF // end_of_file
    }

    struct Token{
        line : u32,
        column : u32,
        token_type : TokenType
    }

    pub fn verify_files(source_files : &[String]) -> Result<(), LexerError>{
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

    pub fn construct_paths(source_files : &[String], path : &str) -> Vec<Box<PathBuf>>{
        return source_files
            .iter()
            .map(|file_name| Box::new(PathBuf::from(path.to_string() + file_name)
            .with_extension(".lexed")))
            .collect();
    }

    pub fn lex_files(source_files : &[String], path : &str) -> Result<(), LexerError>{
        verify_files(source_files)?;
        let output_files : Vec<Box<PathBuf>> = construct_paths(source_files, path);
        for (i, file_name) in source_files.iter().enumerate(){
                let file = match File::open(file_name){
                    Ok(file) => file,
                    Err(e) => {return Err(IOError(file_name.to_owned(), e));}
                };
            let reader = BufReader::new(file);
            for (row, line) in reader.lines().enumerate(){
                let line = match line{
                    Ok(line) => line,
                    Err(e) => {return Err(IOError(file_name.to_owned(), e))}
                };
                for (col, c) in line.chars().enumerate(){
                
                }

            }

        }   
        Ok(())     
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_pass(){
        asse
    }

}
