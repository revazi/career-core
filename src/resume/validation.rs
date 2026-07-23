use super::contract::{
    INPUT_SCHEMA_VERSION, MAX_DOCUMENT_ID_CHARACTERS, MAX_RESUME_LINE_CHARACTERS, MAX_RESUME_LINES,
    MAX_RESUME_TEXT_CHARACTERS, ResumeEvaluationErrorCodeV1, ResumeEvaluationErrorV1,
    ResumeInputV1,
};

pub(crate) struct ValidatedResumeInput<'a> {
    pub(crate) lines: Vec<&'a str>,
}

pub(crate) fn validate_resume_input(
    input: &ResumeInputV1,
) -> Result<ValidatedResumeInput<'_>, ResumeEvaluationErrorV1> {
    if input.schema_version != INPUT_SCHEMA_VERSION {
        return Err(ResumeEvaluationErrorV1::new(
            ResumeEvaluationErrorCodeV1::UnsupportedSchemaVersion,
            format!("schema_version must be {INPUT_SCHEMA_VERSION}."),
            "schema_version",
        ));
    }

    if input.text.trim().is_empty() {
        return Err(ResumeEvaluationErrorV1::new(
            ResumeEvaluationErrorCodeV1::SourceTextEmpty,
            "Resume text must contain non-whitespace characters.",
            "text",
        ));
    }

    let character_count = input.text.chars().count();
    if character_count > MAX_RESUME_TEXT_CHARACTERS {
        return Err(ResumeEvaluationErrorV1::new(
            ResumeEvaluationErrorCodeV1::SourceTextTooLarge,
            format!(
                "Resume text must contain at most {MAX_RESUME_TEXT_CHARACTERS} characters; received {character_count}."
            ),
            "text",
        ));
    }

    let lines = input.text.lines().collect::<Vec<_>>();
    if lines.len() > MAX_RESUME_LINES {
        return Err(ResumeEvaluationErrorV1::new(
            ResumeEvaluationErrorCodeV1::SourceLineCountExceeded,
            format!(
                "Resume text must contain at most {MAX_RESUME_LINES} lines; received {}.",
                lines.len()
            ),
            "text",
        ));
    }

    for (index, line) in lines.iter().enumerate() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let line_character_count = line.chars().count();
        if line_character_count > MAX_RESUME_LINE_CHARACTERS {
            return Err(ResumeEvaluationErrorV1::new(
                ResumeEvaluationErrorCodeV1::SourceLineTooLong,
                format!(
                    "Resume line {} must contain at most {MAX_RESUME_LINE_CHARACTERS} characters; received {line_character_count}.",
                    index + 1
                ),
                format!("text.lines[{}]", index + 1),
            ));
        }
    }

    if let Some(document_id) = &input.metadata.document_id {
        if document_id.trim().is_empty() {
            return Err(ResumeEvaluationErrorV1::new(
                ResumeEvaluationErrorCodeV1::DocumentIdEmpty,
                "metadata.document_id must contain non-whitespace characters when provided.",
                "metadata.document_id",
            ));
        }

        let identifier_character_count = document_id.chars().count();
        if identifier_character_count > MAX_DOCUMENT_ID_CHARACTERS {
            return Err(ResumeEvaluationErrorV1::new(
                ResumeEvaluationErrorCodeV1::DocumentIdTooLong,
                format!(
                    "metadata.document_id must contain at most {MAX_DOCUMENT_ID_CHARACTERS} characters; received {identifier_character_count}."
                ),
                "metadata.document_id",
            ));
        }
    }

    Ok(ValidatedResumeInput { lines })
}
