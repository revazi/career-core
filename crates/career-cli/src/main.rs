#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::process::ExitCode;

use career_core::{Capabilities, CapabilityStatus, capabilities};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "career",
    version,
    about = "Deterministic career-document analysis for people and coding agents"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Report the versioned functionality exposed by this build.
    Capabilities {
        /// Select machine-readable JSON or concise human-readable text.
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Json,
    Text,
}

fn main() -> ExitCode {
    match run(Cli::parse(), &mut io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("career: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli, output: &mut impl Write) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Capabilities { format } => {
            let document = capabilities();
            match format {
                OutputFormat::Json => {
                    serde_json::to_writer_pretty(&mut *output, &document)?;
                    writeln!(output)?;
                }
                OutputFormat::Text => write_capabilities_text(output, &document)?,
            }
        }
    }

    Ok(())
}

fn write_capabilities_text(output: &mut impl Write, document: &Capabilities) -> io::Result<()> {
    writeln!(output, "career-core {}", document.core_version)?;
    writeln!(output, "schema: {}", document.schema_version)?;
    writeln!(output, "deterministic: {}", document.deterministic)?;
    writeln!(
        output,
        "network requests: {}",
        document.performs_network_requests
    )?;
    writeln!(output, "capabilities:")?;

    for capability in &document.capabilities {
        let status = match capability.status {
            CapabilityStatus::Available => "available",
            CapabilityStatus::Planned => "planned",
        };
        writeln!(
            output,
            "  - {} [{}]: {}",
            capability.id, status, capability.summary
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_output_identifies_planned_features() {
        let mut output = Vec::new();
        write_capabilities_text(&mut output, &capabilities()).expect("text output should render");
        let output = String::from_utf8(output).expect("output should be UTF-8");

        assert!(output.contains("core.capabilities [available]"));
        assert!(output.contains("resume.evaluate [planned]"));
        assert!(output.contains("network requests: false"));
    }
}
