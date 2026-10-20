use crate::{
    lox::{LanguageError, Lox},
    parser::ParseError,
};

use std::{
    env,
    fs::File,
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IoError {
    #[error(transparent)]
    BaseIo(#[from] std::io::Error),
    #[error("too many arguments provided. need at most one. received {0}")]
    TooManyArguments(usize),
    #[error("path does not exist: {0}")]
    PathDoesNotExist(String),
    #[error("given path is not a file: {0}")]
    NotFile(String),
}

impl From<IoError> for ExitCode {
    fn from(value: IoError) -> Self {
        let error_code = match value {
            IoError::BaseIo(_)
            | IoError::NotFile(_)
            | IoError::PathDoesNotExist(_) => 66,
            IoError::TooManyArguments(_) => 64,
        };
        ExitCode::from(error_code)
    }
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error(transparent)]
    Language(#[from] LanguageError),
    #[error(transparent)]
    Io(#[from] IoError),
}

impl From<CliError> for ExitCode {
    fn from(value: CliError) -> Self {
        match value {
            CliError::Language(l) => l.into(),
            CliError::Io(i) => i.into(),
        }
    }
}

pub fn run() -> Result<(), CliError> {
    let mut args = env::args();
    let n_args = args.len();
    if n_args > 2 {
        return Err(IoError::TooManyArguments(n_args - 1).into());
    }
    let cmd_name = args.next();
    assert!(cmd_name.is_some());

    if let Some(file_path) = args.next() {
        let path = PathBuf::from(file_path);
        run_file(&path)
    } else {
        run_prompt()
    }
}

fn open_file(path: &Path) -> Result<File, IoError> {
    File::open(path).map_err(IoError::BaseIo)
}

fn run_file(path: &Path) -> Result<(), CliError> {
    if !path.exists() {
        return Err(IoError::PathDoesNotExist(
            path.to_string_lossy().to_string(),
        )
        .into());
    }
    if !path.is_file() {
        return Err(IoError::NotFile(path.to_string_lossy().to_string()).into());
    }
    println!("running file at {path:?}");
    let file = open_file(path)?;
    let mut reader = BufReader::new(file);

    let mut program = Lox::new(false)?;
    let line_count = 1;

    let mut lines = String::new();
    reader.read_to_string(&mut lines).map_err(IoError::BaseIo)?;

    program.run(&lines, &line_count)?;
    Ok(())
}

fn run_prompt() -> Result<(), CliError> {
    let mut program = Lox::new(true)?;
    let mut input = String::new();
    println!("starting lox interpreter");
    let mut line_count = 1;
    let mut saved = String::new();
    let mut in_block = false;
    loop {
        if !in_block {
            print!("> ");
            saved = String::new();
        }
        io::stdout().flush().map_err(IoError::BaseIo)?;
        io::stdin().read_line(&mut input).map_err(IoError::BaseIo)?;
        let cleaned_input = input.trim();
        if cleaned_input.is_empty() {
            if in_block {
                in_block = false;
                continue;
            }
            break;
        }
        saved = saved + &input;
        match program.run(&saved, &line_count) {
            Ok(()) => {
                in_block = false;
            }
            Err(LanguageError::Parse(p))
                if p.len() == 1
                    && p.first()
                        .map(|e| {
                            matches!(
                                e,
                                ParseError::UnterminatedBlock(_)
                                    | ParseError::UnterminatedCall(_)
                            )
                        })
                        .unwrap_or(false) =>
            {
                in_block = true;
            }
            Err(e) => {
                in_block = false;
                println!("language error in above line: {e}");
            }
        }
        input.clear();
        line_count += 1;
    }
    Ok(())
}
