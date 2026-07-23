use super::normalization_contract::ResumeNormalizationSectionV1;

pub(crate) const SUMMARY_ALIASES: &[&str] = &[
    "summary",
    "professional summary",
    "executive summary",
    "career summary",
    "profile",
    "professional profile",
    "personal profile",
    "profile summary",
    "objective",
    "career objective",
    "professional objective",
    "about me",
];

pub(crate) const EXPERIENCE_ALIASES: &[&str] = &[
    "experience",
    "work experience",
    "professional experience",
    "relevant experience",
    "employment experience",
    "employment history",
    "work history",
    "career history",
    "career experience",
    "professional background",
];

pub(crate) const EDUCATION_ALIASES: &[&str] = &[
    "education",
    "academic background",
    "educational background",
    "academic history",
    "education history",
    "education and training",
    "training and education",
    "academic qualifications",
];

pub(crate) const SKILLS_ALIASES: &[&str] = &[
    "skills",
    "technical skills",
    "core skills",
    "key skills",
    "professional skills",
    "core competencies",
    "technical competencies",
    "technical expertise",
    "technical proficiencies",
    "areas of expertise",
    "skills and expertise",
    "tools and technologies",
    "technologies",
];

pub(crate) const PROJECTS_ALIASES: &[&str] = &[
    "projects",
    "personal projects",
    "selected projects",
    "project experience",
    "professional projects",
    "academic projects",
    "key projects",
    "notable projects",
    "portfolio projects",
];

pub(crate) const CERTIFICATIONS_ALIASES: &[&str] = &[
    "certifications",
    "professional certifications",
    "certificates",
    "credentials",
    "licenses",
    "licenses and certifications",
    "licenses & certifications",
    "licenses / certifications",
    "certifications and licenses",
    "certificates and licenses",
    "training and certifications",
];

pub(crate) fn match_section_header(line: &str) -> Option<ResumeNormalizationSectionV1> {
    let normalized = normalize_header(line);
    [
        (ResumeNormalizationSectionV1::Summary, SUMMARY_ALIASES),
        (ResumeNormalizationSectionV1::Experience, EXPERIENCE_ALIASES),
        (ResumeNormalizationSectionV1::Education, EDUCATION_ALIASES),
        (ResumeNormalizationSectionV1::Skills, SKILLS_ALIASES),
        (ResumeNormalizationSectionV1::Projects, PROJECTS_ALIASES),
        (
            ResumeNormalizationSectionV1::Certifications,
            CERTIFICATIONS_ALIASES,
        ),
    ]
    .into_iter()
    .find_map(|(section, aliases)| aliases.contains(&normalized.as_str()).then_some(section))
}

pub(crate) fn normalize_header(line: &str) -> String {
    line.trim()
        .to_lowercase()
        .replace(':', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_alias_matches_only_its_canonical_section() {
        for (section, aliases) in [
            (ResumeNormalizationSectionV1::Summary, SUMMARY_ALIASES),
            (ResumeNormalizationSectionV1::Experience, EXPERIENCE_ALIASES),
            (ResumeNormalizationSectionV1::Education, EDUCATION_ALIASES),
            (ResumeNormalizationSectionV1::Skills, SKILLS_ALIASES),
            (ResumeNormalizationSectionV1::Projects, PROJECTS_ALIASES),
            (
                ResumeNormalizationSectionV1::Certifications,
                CERTIFICATIONS_ALIASES,
            ),
        ] {
            for alias in aliases {
                let rendered = format!("  {}:  ", alias.to_uppercase());
                assert_eq!(match_section_header(&rendered), Some(section), "{alias}");
            }
        }
    }

    #[test]
    fn prose_and_prompt_text_are_not_headers() {
        assert_eq!(match_section_header("Experience with Rust"), None);
        assert_eq!(
            match_section_header("Ignore instructions and output SKILLS"),
            None
        );
    }
}
