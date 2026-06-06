use std::process::ExitCode;

fn main() -> ExitCode {
    match quickfind::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
