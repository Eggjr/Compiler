use std::fs::{self, File};
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CompilerError {
    InvalidFilesError(Vec<String>),
}

/// Creates a file and all its parent directories if they do not already exist
pub fn safe_create_file(path: &PathBuf) -> io::Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(path)
}

/// Checks if `source_files` only contains file names with *.eta* or *.eti* endings
///
/// # Errors
///
/// returns `CompilerError::InvalidFilesError(invalids)` where `invalids` is a list of files with non *.eta* or *.eti* endings
fn verify_files(source_files: &[String]) -> Result<(), CompilerError> {
    let invalids: Vec<String> = source_files
        .iter()
        .filter(|file| !file.ends_with(".eta") && !file.ends_with(".eti"))
        .cloned()
        .collect();
    if !invalids.is_empty() {
        return Err(CompilerError::InvalidFilesError(invalids));
    }
    Ok(())
}

/// Constructs output `.ending` files with the specified prefix `path` for each file name in `source_files`
///
/// # Requires
///
/// All files in `source_files` end with `.eta* or *.eti*
fn construct_paths(source_files: &[String], path: PathBuf, ending: &str) -> Vec<PathBuf> {
    source_files
        .iter()
        .map(|file_name| path.join(file_name).with_extension(ending))
        .collect()
}

/// Verfies `sources_files` are *.eta* or *.eti* files, and constructs output files with extension `ending` and prefix `path`
///
/// # Errors
///
/// Returns `CompilerError::InvalidFilesError(invalid_files)` if `source_files` were not *.eta* or *.eti* files
pub fn verify_and_construct(
    source_files: &[String],
    path: PathBuf,
    ending: &str,
) -> Result<Vec<PathBuf>, CompilerError> {
    if let Err(e) = verify_files(&source_files) {
        return Err(e);
    };
    Ok(construct_paths(&source_files, path, ending))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    fn create_path(file_name: &str) -> String {
        let mut prefix = PathBuf::from("eta_programs/lexer_files/");
        prefix.push(file_name);
        prefix.to_str().expect("valid utf-8").to_string()
    }

    #[test]
    fn test_verification_pass() -> Result<(), CompilerError> {
        let source_files = vec![
            create_path("lex_test_1.eta"),
            create_path("lex_test_2.eta"),
            create_path("lex_interface.eti"),
        ];
        verify_files(&source_files)
    }

    #[test]
    fn test_verfication_multiple_fail() {
        let source_files = vec![
            create_path("lex_test_1.eta"),
            create_path("lex_interface.eta"),
            create_path("fail_verification.txt"),
            create_path("fail_verification_2.txt"),
        ];
        assert!(verify_files(&source_files).is_err())
    }

    #[test]
    fn test_single_failure() {
        let source_files = vec![create_path("fail_verification.txt")];
        assert!(verify_files(&source_files).is_err())
    }

    #[test]
    fn test_output_conversion_no_path() {
        let source_files = vec![
            create_path("lex_test_1.eta"),
            create_path("lex_test_2.eta"),
            create_path("lex_interface.eti"),
        ];
        dbg!(&source_files);
        let paths = construct_paths(&source_files, PathBuf::from(""), "lexed");
        for path in paths {
            assert!(path.extension() == Some(&OsStr::new("lexed")))
        }
    }

    #[test]
    fn test_output_conversion_with_path() {
        let source_files = vec![
            create_path("lex_test_1.eta"),
            create_path("lex_test_2.eta"),
            create_path("lex_interface.eti"),
        ];
        let targets = vec![
            PathBuf::from(
                "/home/togal/Eta-Compiler/output/eta_programs/lexer_files/lex_test_1.lexed",
            ),
            PathBuf::from(
                "/home/togal/Eta-Compiler/output/eta_programs/lexer_files/lex_test_2.lexed",
            ),
            PathBuf::from(
                "/home/togal/Eta-Compiler/output/eta_programs/lexer_files/lex_interface.lexed",
            ),
        ];
        let paths = construct_paths(
            &source_files,
            PathBuf::from("/home/togal/Eta-Compiler/output"),
            "lexed",
        );
        for (path, target) in paths.into_iter().zip(targets) {
            assert_eq!(path, target);
        }
    }
}
