#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process;
use std::thread;
use std::time::Duration;

fn write_hex_argument(argument: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    for byte in argument.as_bytes() {
        write!(stdout, "{byte:02x}")?;
    }
    writeln!(stdout)
}

fn main() -> io::Result<()> {
    let mut arguments = env::args().skip(1);
    match arguments.next().as_deref() {
        Some("--test-argv") => {
            for argument in arguments {
                write_hex_argument(&argument)?;
            }
        }
        Some("--test-stdio-exit") => {
            let exit_code = arguments
                .next()
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(91);
            let mut input = Vec::new();
            io::stdin().read_to_end(&mut input)?;
            io::stdout().write_all(b"stdout:")?;
            io::stdout().write_all(&input)?;
            io::stderr().write_all(b"stderr:")?;
            io::stderr().write_all(&input)?;
            process::exit(exit_code);
        }
        Some("--test-signal") => {
            if let Ok(path) = env::var("CAREER_NPM_HELPER_PID_FILE") {
                fs::write(path, process::id().to_string())?;
            }
            writeln!(io::stdout(), "READY")?;
            io::stdout().flush()?;
            thread::sleep(Duration::from_secs(60));
        }
        _ => process::exit(92),
    }
    Ok(())
}
