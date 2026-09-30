use crate::{
    error::{CliError, LanguageError},
    lox::Lox,
};
use std::{
    env,
    fs::File,
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
};

pub fn run() -> Result<(), CliError> {
    let mut args = env::args();
    let n_args = args.len();
    if n_args > 2 {
        return Err(CliError::Parse(
            "too many arguments provided. need at most one.",
        ));
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

fn run_file(path: &Path) -> Result<(), CliError> {
    if !path.exists() {
        return Err(CliError::PathDoesNotExist(
            path.to_string_lossy().to_string(),
        ));
    }
    if !path.is_file() {
        return Err(CliError::NotFile(path.to_string_lossy().to_string()));
    }
    println!("running file at {path:?}");
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut program = Lox::new(false);
    let line_count = 1;

    let mut lines = String::new();
    reader.read_to_string(&mut lines)?;

    program.run(&lines, &line_count).map_err(|e| match e {
        LanguageError::Runtime(r) => CliError::Runtime {
            line: line_count,
            error: r,
        },
        _ => CliError::Language {
            line: line_count,
            error: e,
        },
    })?;
    Ok(())
}

fn run_prompt() -> Result<(), CliError> {
    let mut program = Lox::new(true);
    let mut input = String::new();
    println!("starting lox interpreter");
    let mut line_count = 1;
    loop {
        print!("> ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        let cleaned_input = input.trim();
        if cleaned_input.is_empty() {
            break;
        }
        match program.run(&input, &line_count) {
            Ok(()) => {}
            Err(e) => {
                println!("language error in above line: {e}");
            }
        }
        input.clear();
        line_count += 1;
    }
    Ok(())
}
