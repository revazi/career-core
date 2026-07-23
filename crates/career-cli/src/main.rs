#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use career_core::{
    Capabilities, CapabilityStatus, CareerErrorV1, ERROR_SCHEMA_VERSION, JobFieldDetectionStatusV1,
    JobInputV1, JobNormalizationV1, JobParseConfidenceLabelV1, ResumeAnalysisCategoryV1,
    ResumeAnalysisFindingStatusV1, ResumeAnalysisV1, ResumeDetectionStatusV1,
    ResumeEnrichmentInputV1, ResumeEnrichmentMergeStatusV1, ResumeEnrichmentResultV1,
    ResumeEvaluationV1, ResumeFieldDetectionStatusV1, ResumeInputV1, ResumeNormalizationV1,
    ResumeParseConfidenceLabelV1, analyze_resume, apply_resume_enrichment, capabilities,
    evaluate_resume, normalize_job, normalize_resume,
};
use clap::{Parser, Subcommand, ValueEnum, error::ErrorKind};
use serde_json::json;

const MAX_CLI_INPUT_BYTES: usize = 262_144;
const EXIT_INPUT_IO: u8 = 3;
const EXIT_INVALID_JSON: u8 = 4;
const EXIT_INVALID_INPUT: u8 = 5;
const EXIT_OUTPUT_FAILURE: u8 = 6;

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
    /// Evaluate, analyze, normalize, or enrich resume input.
    Resume {
        #[command(subcommand)]
        command: ResumeCommand,
    },
    /// Normalize job-description input.
    Job {
        #[command(subcommand)]
        command: JobCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ResumeCommand {
    /// Evaluate recognized core-section coverage from versioned JSON input.
    Evaluate {
        /// Read input JSON from this path, or use `-` for stdin.
        #[arg(long)]
        input: PathBuf,
        /// Select machine-readable JSON or concise human-readable text.
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// Analyze resume readiness with the full deterministic scoring policy.
    Analyze {
        /// Read career.resume_input.v1 JSON from this path, or use `-` for stdin.
        #[arg(long)]
        input: PathBuf,
        /// Select machine-readable JSON or concise human-readable text.
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// Normalize bounded resume text into deterministic source-grounded facts.
    Normalize {
        /// Read career.resume_input.v1 JSON from this path, or use `-` for stdin.
        #[arg(long)]
        input: PathBuf,
        /// Select machine-readable JSON or concise human-readable text.
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// Validate and merge an explicit external proposal without making a network request.
    Enrich {
        /// Read career.resume_enrichment_input.v1 JSON from this path, or use `-` for stdin.
        #[arg(long)]
        input: PathBuf,
        /// Select machine-readable JSON or concise human-readable text.
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
enum JobCommand {
    /// Normalize bounded job-description text into deterministic source-grounded facts.
    Normalize {
        /// Read career.job_input.v1 JSON from this path, or use `-` for stdin.
        #[arg(long)]
        input: PathBuf,
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

#[derive(Debug)]
enum CliFailure {
    Adapter {
        code: &'static str,
        message: String,
        field_path: Option<&'static str>,
        exit_code: u8,
    },
    Core(CareerErrorV1),
}

impl CliFailure {
    fn invalid_arguments() -> Self {
        Self::Adapter {
            code: "invalid_arguments",
            message: "Command arguments were invalid. Run `career --help` for usage.".to_owned(),
            field_path: None,
            exit_code: 2,
        }
    }

    fn input_read_failed(document_kind: &'static str) -> Self {
        Self::Adapter {
            code: "input_read_failed",
            message: format!("Could not read {document_kind} input."),
            field_path: Some("input"),
            exit_code: EXIT_INPUT_IO,
        }
    }

    fn cli_input_too_large(actual_bytes: usize) -> Self {
        Self::Adapter {
            code: "cli_input_too_large",
            message: format!(
                "CLI input must contain at most {MAX_CLI_INPUT_BYTES} bytes; received more than that limit ({actual_bytes} bytes read)."
            ),
            field_path: Some("input"),
            exit_code: EXIT_INPUT_IO,
        }
    }

    fn invalid_json(error: &serde_json::Error, expected_contract: &str) -> Self {
        Self::Adapter {
            code: "invalid_json",
            message: format!(
                "Input must be valid {expected_contract} JSON (line {}, column {}).",
                error.line(),
                error.column()
            ),
            field_path: Some("input"),
            exit_code: EXIT_INVALID_JSON,
        }
    }

    fn output_failure() -> Self {
        Self::Adapter {
            code: "output_write_failed",
            message: "Could not write command output.".to_owned(),
            field_path: None,
            exit_code: EXIT_OUTPUT_FAILURE,
        }
    }

    fn exit_code(&self) -> u8 {
        match self {
            Self::Adapter { exit_code, .. } => *exit_code,
            Self::Core(_) => EXIT_INVALID_INPUT,
        }
    }

    fn report(&self, format: OutputFormat, error_output: &mut impl Write) {
        let result = match (self, format) {
            (
                Self::Adapter {
                    code,
                    message,
                    field_path,
                    ..
                },
                OutputFormat::Json,
            ) => {
                let document = json!({
                    "schema_version": ERROR_SCHEMA_VERSION,
                    "code": code,
                    "message": message,
                    "field_path": field_path,
                });
                serde_json::to_writer_pretty(&mut *error_output, &document)
                    .and_then(|()| writeln!(error_output).map_err(serde_json::Error::io))
            }
            (Self::Core(error), OutputFormat::Json) => {
                serde_json::to_writer_pretty(&mut *error_output, error)
                    .and_then(|()| writeln!(error_output).map_err(serde_json::Error::io))
            }
            (
                Self::Adapter {
                    code,
                    message,
                    field_path,
                    ..
                },
                OutputFormat::Text,
            ) => writeln!(
                error_output,
                "career [{code}]: {message}{}",
                field_path
                    .map(|path| format!(" (field: {path})"))
                    .unwrap_or_default()
            )
            .map_err(serde_json::Error::io),
            (Self::Core(error), OutputFormat::Text) => writeln!(
                error_output,
                "career [{}]: {} (field: {})",
                error.code.as_str(),
                error.message,
                error.field_path
            )
            .map_err(serde_json::Error::io),
        };

        let _ = result;
    }
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            return if error.print().is_ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(EXIT_OUTPUT_FAILURE)
            };
        }
        Err(_) => {
            let error = CliFailure::invalid_arguments();
            error.report(OutputFormat::Json, &mut io::stderr().lock());
            return ExitCode::from(error.exit_code());
        }
    };
    let output_format = cli.output_format();
    let result = run(cli, &mut io::stdin().lock(), &mut io::stdout().lock());

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let exit_code = error.exit_code();
            error.report(output_format, &mut io::stderr().lock());
            ExitCode::from(exit_code)
        }
    }
}

impl Cli {
    fn output_format(&self) -> OutputFormat {
        match &self.command {
            Command::Capabilities { format } => *format,
            Command::Resume { command } => match command {
                ResumeCommand::Evaluate { format, .. }
                | ResumeCommand::Analyze { format, .. }
                | ResumeCommand::Normalize { format, .. }
                | ResumeCommand::Enrich { format, .. } => *format,
            },
            Command::Job { command } => match command {
                JobCommand::Normalize { format, .. } => *format,
            },
        }
    }
}

fn run(
    cli: Cli,
    standard_input: &mut impl Read,
    output: &mut impl Write,
) -> Result<(), CliFailure> {
    match cli.command {
        Command::Capabilities { format } => {
            let document = capabilities();
            match format {
                OutputFormat::Json => write_json(output, &document),
                OutputFormat::Text => write_capabilities_text(output, &document),
            }
        }
        Command::Resume {
            command: ResumeCommand::Evaluate { input, format },
        } => {
            let input_bytes = read_input(&input, standard_input, "resume")?;
            let resume_input = serde_json::from_slice::<ResumeInputV1>(&input_bytes)
                .map_err(|error| CliFailure::invalid_json(&error, "career.resume_input.v1"))?;
            let evaluation = evaluate_resume(&resume_input).map_err(CliFailure::Core)?;
            match format {
                OutputFormat::Json => write_json(output, &evaluation),
                OutputFormat::Text => write_resume_evaluation_text(output, &evaluation),
            }
        }
        Command::Resume {
            command: ResumeCommand::Analyze { input, format },
        } => {
            let input_bytes = read_input(&input, standard_input, "resume")?;
            let resume_input = serde_json::from_slice::<ResumeInputV1>(&input_bytes)
                .map_err(|error| CliFailure::invalid_json(&error, "career.resume_input.v1"))?;
            let analysis = analyze_resume(&resume_input).map_err(CliFailure::Core)?;
            match format {
                OutputFormat::Json => write_json(output, &analysis),
                OutputFormat::Text => write_resume_analysis_text(output, &analysis),
            }
        }
        Command::Resume {
            command: ResumeCommand::Normalize { input, format },
        } => {
            let input_bytes = read_input(&input, standard_input, "resume")?;
            let resume_input = serde_json::from_slice::<ResumeInputV1>(&input_bytes)
                .map_err(|error| CliFailure::invalid_json(&error, "career.resume_input.v1"))?;
            let normalization = normalize_resume(&resume_input).map_err(CliFailure::Core)?;
            match format {
                OutputFormat::Json => write_json(output, &normalization),
                OutputFormat::Text => write_resume_normalization_text(output, &normalization),
            }
        }
        Command::Resume {
            command: ResumeCommand::Enrich { input, format },
        } => {
            let input_bytes = read_input(&input, standard_input, "resume")?;
            let enrichment_input = serde_json::from_slice::<ResumeEnrichmentInputV1>(&input_bytes)
                .map_err(|error| {
                    CliFailure::invalid_json(&error, "career.resume_enrichment_input.v1")
                })?;
            let result = apply_resume_enrichment(&enrichment_input).map_err(CliFailure::Core)?;
            match format {
                OutputFormat::Json => write_json(output, &result),
                OutputFormat::Text => write_resume_enrichment_text(output, &result),
            }
        }
        Command::Job {
            command: JobCommand::Normalize { input, format },
        } => {
            let input_bytes = read_input(&input, standard_input, "job-description")?;
            let job_input = serde_json::from_slice::<JobInputV1>(&input_bytes)
                .map_err(|error| CliFailure::invalid_json(&error, "career.job_input.v1"))?;
            let normalization = normalize_job(&job_input).map_err(CliFailure::Core)?;
            match format {
                OutputFormat::Json => write_json(output, &normalization),
                OutputFormat::Text => write_job_normalization_text(output, &normalization),
            }
        }
    }
}

fn read_input(
    path: &PathBuf,
    standard_input: &mut impl Read,
    document_kind: &'static str,
) -> Result<Vec<u8>, CliFailure> {
    if path.as_os_str() == "-" {
        read_bounded(standard_input, document_kind)
    } else {
        let mut file =
            File::open(path).map_err(|_| CliFailure::input_read_failed(document_kind))?;
        read_bounded(&mut file, document_kind)
    }
}

fn read_bounded(
    reader: &mut impl Read,
    document_kind: &'static str,
) -> Result<Vec<u8>, CliFailure> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_CLI_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| CliFailure::input_read_failed(document_kind))?;

    if bytes.len() > MAX_CLI_INPUT_BYTES {
        return Err(CliFailure::cli_input_too_large(bytes.len()));
    }

    Ok(bytes)
}

fn write_json(output: &mut impl Write, document: &impl serde::Serialize) -> Result<(), CliFailure> {
    serde_json::to_writer_pretty(&mut *output, document)
        .map_err(|_| CliFailure::output_failure())?;
    writeln!(output).map_err(|_| CliFailure::output_failure())
}

fn write_capabilities_text(
    output: &mut impl Write,
    document: &Capabilities,
) -> Result<(), CliFailure> {
    writeln!(output, "career-core {}", document.core_version)
        .map_err(|_| CliFailure::output_failure())?;
    writeln!(output, "schema: {}", document.schema_version)
        .map_err(|_| CliFailure::output_failure())?;
    writeln!(output, "deterministic: {}", document.deterministic)
        .map_err(|_| CliFailure::output_failure())?;
    writeln!(
        output,
        "network requests: {}",
        document.performs_network_requests
    )
    .map_err(|_| CliFailure::output_failure())?;
    writeln!(output, "capabilities:").map_err(|_| CliFailure::output_failure())?;

    for capability in &document.capabilities {
        let status = match capability.status {
            CapabilityStatus::Available => "available",
            CapabilityStatus::Planned => "planned",
        };
        writeln!(
            output,
            "  - {} [{}]: {}",
            capability.id, status, capability.summary
        )
        .map_err(|_| CliFailure::output_failure())?;
    }

    Ok(())
}

fn write_job_normalization_text(
    output: &mut impl Write,
    normalization: &JobNormalizationV1,
) -> Result<(), CliFailure> {
    writeln!(
        output,
        "Job-description normalization: {} confidence ({}/100)",
        job_confidence_label(normalization.confidence.label),
        normalization.confidence.score
    )
    .map_err(|_| CliFailure::output_failure())?;
    for field in &normalization.field_statuses {
        let status = match field.status {
            JobFieldDetectionStatusV1::Detected => "detected",
            JobFieldDetectionStatusV1::NotDetected => "not detected",
        };
        writeln!(output, "  - {}: {status}", field.field.label())
            .map_err(|_| CliFailure::output_failure())?;
    }
    writeln!(
        output,
        "Required skills: {}; preferred skills: {}; responsibilities: {}",
        normalization.deterministic_document.required_skills.len(),
        normalization.deterministic_document.preferred_skills.len(),
        normalization.deterministic_document.responsibilities.len()
    )
    .map_err(|_| CliFailure::output_failure())?;
    for warning in &normalization.warnings {
        writeln!(output, "Warning: {}", warning.message)
            .map_err(|_| CliFailure::output_failure())?;
    }
    Ok(())
}

fn job_confidence_label(label: JobParseConfidenceLabelV1) -> &'static str {
    match label {
        JobParseConfidenceLabelV1::Unknown => "unknown",
        JobParseConfidenceLabelV1::Low => "low",
        JobParseConfidenceLabelV1::Medium => "medium",
        JobParseConfidenceLabelV1::High => "high",
    }
}

fn write_resume_evaluation_text(
    output: &mut impl Write,
    evaluation: &ResumeEvaluationV1,
) -> Result<(), CliFailure> {
    writeln!(
        output,
        "Resume section-coverage evaluation: {}/100",
        evaluation.score
    )
    .map_err(|_| CliFailure::output_failure())?;

    for check in &evaluation.checks {
        let status = match check.status {
            ResumeDetectionStatusV1::DetectedWithContent => "detected with content",
            ResumeDetectionStatusV1::DetectedWithoutContent => "detected without content",
            ResumeDetectionStatusV1::NotDetected => "not detected",
        };
        writeln!(
            output,
            "  - {}: {} ({}/100)",
            check.section.label(),
            status,
            check.score
        )
        .map_err(|_| CliFailure::output_failure())?;
    }

    writeln!(output, "Warnings:").map_err(|_| CliFailure::output_failure())?;
    for warning in &evaluation.warnings {
        writeln!(output, "  - {}", warning.message).map_err(|_| CliFailure::output_failure())?;
    }

    Ok(())
}

fn write_resume_analysis_text(
    output: &mut impl Write,
    analysis: &ResumeAnalysisV1,
) -> Result<(), CliFailure> {
    writeln!(
        output,
        "Deterministic resume analysis: {}/100",
        analysis.overall_score
    )
    .map_err(|_| CliFailure::output_failure())?;
    writeln!(
        output,
        "Parser confidence: {} ({}/100)",
        parse_confidence_label(analysis.confidence_context.parse_confidence.label),
        analysis.confidence_context.parse_confidence.score
    )
    .map_err(|_| CliFailure::output_failure())?;
    writeln!(output, "Categories:").map_err(|_| CliFailure::output_failure())?;
    for category in ResumeAnalysisCategoryV1::ALL {
        writeln!(
            output,
            "  - {}: {}/100",
            analysis_category_label(category),
            analysis.category_scores.get(category)
        )
        .map_err(|_| CliFailure::output_failure())?;
    }
    if !analysis.top_strengths.is_empty() {
        writeln!(output, "Top strengths:").map_err(|_| CliFailure::output_failure())?;
        for strength in &analysis.top_strengths {
            writeln!(output, "  - {}: {}", strength.title, strength.reason)
                .map_err(|_| CliFailure::output_failure())?;
        }
    }
    if !analysis.top_weaknesses.is_empty() {
        writeln!(output, "Top weaknesses:").map_err(|_| CliFailure::output_failure())?;
        for weakness in &analysis.top_weaknesses {
            let status = match weakness.status {
                ResumeAnalysisFindingStatusV1::Confirmed => "confirmed",
                ResumeAnalysisFindingStatusV1::Provisional => "provisional",
            };
            writeln!(
                output,
                "  - {} [{status}]: {}",
                weakness.title, weakness.reason
            )
            .map_err(|_| CliFailure::output_failure())?;
        }
    }
    if !analysis.improvement_actions.is_empty() {
        writeln!(output, "Improvement actions:").map_err(|_| CliFailure::output_failure())?;
        for action in &analysis.improvement_actions {
            writeln!(output, "  {}. {}", action.priority, action.action)
                .map_err(|_| CliFailure::output_failure())?;
        }
    }
    for warning in &analysis.warnings {
        writeln!(output, "Warning: {}", warning.message)
            .map_err(|_| CliFailure::output_failure())?;
    }
    Ok(())
}

fn analysis_category_label(category: ResumeAnalysisCategoryV1) -> &'static str {
    match category {
        ResumeAnalysisCategoryV1::FormatAts => "ATS readability signals",
        ResumeAnalysisCategoryV1::ContentStrength => "Content strength",
        ResumeAnalysisCategoryV1::ExperienceImpact => "Experience impact",
        ResumeAnalysisCategoryV1::SkillsCoverage => "Skills coverage",
        ResumeAnalysisCategoryV1::Presentation => "Presentation",
        ResumeAnalysisCategoryV1::Completeness => "Completeness",
    }
}

fn parse_confidence_label(label: ResumeParseConfidenceLabelV1) -> &'static str {
    match label {
        ResumeParseConfidenceLabelV1::Unknown => "unknown",
        ResumeParseConfidenceLabelV1::Low => "low",
        ResumeParseConfidenceLabelV1::Medium => "medium",
        ResumeParseConfidenceLabelV1::High => "high",
    }
}

fn write_resume_normalization_text(
    output: &mut impl Write,
    normalization: &ResumeNormalizationV1,
) -> Result<(), CliFailure> {
    let confidence = parse_confidence_label(normalization.confidence.label);
    writeln!(
        output,
        "Resume normalization: {confidence} confidence ({}/100)",
        normalization.confidence.score
    )
    .map_err(|_| CliFailure::output_failure())?;
    for field in &normalization.field_statuses {
        let status = match field.status {
            ResumeFieldDetectionStatusV1::Detected => "detected",
            ResumeFieldDetectionStatusV1::LikelyMissing => "likely missing",
            ResumeFieldDetectionStatusV1::NotDetected => "not detected",
        };
        writeln!(output, "  - {}: {status}", field.field.label())
            .map_err(|_| CliFailure::output_failure())?;
    }
    writeln!(
        output,
        "External enrichment: {} ({})",
        normalization.enrichment_request.status.label(),
        normalization.enrichment_request.reason.label()
    )
    .map_err(|_| CliFailure::output_failure())?;
    if !normalization.enrichment_request.target_sections.is_empty() {
        writeln!(
            output,
            "  targets: {}",
            normalization
                .enrichment_request
                .target_sections
                .iter()
                .map(|section| section.label())
                .collect::<Vec<_>>()
                .join(", ")
        )
        .map_err(|_| CliFailure::output_failure())?;
    }
    for warning in &normalization.warnings {
        writeln!(output, "Warning: {}", warning.message)
            .map_err(|_| CliFailure::output_failure())?;
    }
    Ok(())
}

fn write_resume_enrichment_text(
    output: &mut impl Write,
    result: &ResumeEnrichmentResultV1,
) -> Result<(), CliFailure> {
    let status = match result.merge.status {
        ResumeEnrichmentMergeStatusV1::Applied => "applied",
        ResumeEnrichmentMergeStatusV1::NotApplied => "not applied",
    };
    writeln!(output, "Resume external enrichment: {status}")
        .map_err(|_| CliFailure::output_failure())?;
    writeln!(
        output,
        "Deterministic confidence preserved: {} ({}/100)",
        parse_confidence_label(result.baseline.confidence.label),
        result.baseline.confidence.score
    )
    .map_err(|_| CliFailure::output_failure())?;
    for field in &result.merge.field_provenance {
        writeln!(
            output,
            "  - {}: {}",
            field.field.label(),
            field.source.label()
        )
        .map_err(|_| CliFailure::output_failure())?;
    }
    for warning in &result.warnings {
        writeln!(output, "Warning: {}", warning.message)
            .map_err(|_| CliFailure::output_failure())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_output_identifies_available_and_planned_features() {
        let mut output = Vec::new();
        write_capabilities_text(&mut output, &capabilities()).expect("text output should render");
        let output = String::from_utf8(output).expect("output should be UTF-8");

        assert!(output.contains("core.capabilities [available]"));
        assert!(output.contains("resume.evaluate [available]"));
        assert!(output.contains("resume.analyze [available]"));
        assert!(output.contains("resume.normalize [available]"));
        assert!(output.contains("resume.enrich [available]"));
        assert!(output.contains("job.normalize [available]"));
        assert!(output.contains("job.match [planned]"));
        assert!(output.contains("network requests: false"));
    }
}
