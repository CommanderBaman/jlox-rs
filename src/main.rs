use std::process::ExitCode;

fn main() -> ExitCode {
    match jlox_rs::run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            println!("some error occurred. details:\n{}", e);
            ExitCode::from(e)
        }
    }
}
