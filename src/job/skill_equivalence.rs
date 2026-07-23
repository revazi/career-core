use std::collections::BTreeMap;
use std::sync::OnceLock;

const SKILL_EQUIVALENCE_GROUPS: &[&[&str]] = &[
    &["aws", "amazon aws", "amazon web services"],
    &["gcp", "google cloud", "google cloud platform"],
    &["javascript", "js"],
    &["typescript", "ts"],
    &["postgresql", "postgres"],
    &["kubernetes", "k8s"],
    &["node", "node js", "nodejs"],
    &["react", "react js", "reactjs"],
    &["vue", "vue js", "vuejs"],
    &["next js", "nextjs"],
    &["dotnet", "net", ".net"],
    &["c#", "c sharp", "csharp"],
    &["c++", "c plus plus", "cpp"],
    &["golang", "go"],
    &["machine learning", "ml"],
    &["artificial intelligence", "ai"],
    &[
        "ci cd",
        "continuous integration continuous delivery",
        "continuous integration and continuous delivery",
    ],
];

static SKILL_ALIASES: OnceLock<BTreeMap<String, String>> = OnceLock::new();

/// Returns true only for equal canonical names, explicit reviewed aliases, or
/// compatible versioned/unversioned forms of the same technology.
#[must_use]
pub fn are_conservative_skill_equivalents(job_skill: &str, resume_skill: &str) -> bool {
    let (job_key, job_version) = canonicalize_skill_with_version(job_skill);
    let (resume_key, resume_version) = canonicalize_skill_with_version(resume_skill);
    if job_key.is_empty() || job_key != resume_key {
        return false;
    }
    if !job_version.is_empty() && !resume_version.is_empty() && job_version != resume_version {
        return false;
    }
    true
}

/// Canonicalizes a skill with the reviewed alias policy while omitting a
/// trailing explicit version.
#[must_use]
pub fn canonicalize_skill(value: &str) -> String {
    canonicalize_skill_with_version(value).0
}

pub(crate) fn normalize_skill_name(value: &str) -> String {
    let mut normalized = String::new();
    for character in value.chars() {
        if character.is_alphanumeric() || matches!(character, '+' | '#') {
            for lowercase in character.to_lowercase() {
                normalized.push(lowercase);
            }
        } else {
            normalized.push(' ');
        }
    }
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn canonicalize_skill_with_version(value: &str) -> (String, Vec<String>) {
    let normalized = normalize_skill_name(value);
    if normalized.is_empty() {
        return (String::new(), Vec::new());
    }
    let (versionless, version) = split_trailing_version(&normalized);
    let canonical = skill_aliases()
        .get(&versionless)
        .cloned()
        .unwrap_or(versionless);
    (canonical, version)
}

fn split_trailing_version(value: &str) -> (String, Vec<String>) {
    let mut tokens = value
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut version = Vec::new();
    while tokens.len() > 1 && tokens.last().is_some_and(|token| is_version_token(token)) {
        if let Some(token) = tokens.pop() {
            version.push(token.trim_start_matches('v').to_owned());
        }
    }
    version.reverse();
    (tokens.join(" "), version)
}

fn is_version_token(value: &str) -> bool {
    value.chars().all(|character| character.is_ascii_digit())
        || value
            .strip_prefix('v')
            .is_some_and(|suffix| !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()))
}

fn skill_aliases() -> &'static BTreeMap<String, String> {
    SKILL_ALIASES.get_or_init(|| {
        let mut aliases = BTreeMap::new();
        for group in SKILL_EQUIVALENCE_GROUPS {
            let canonical = group
                .first()
                .map(|value| normalize_skill_name(value))
                .unwrap_or_default();
            for value in *group {
                aliases.insert(normalize_skill_name(value), canonical.clone());
            }
        }
        aliases
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_reviewed_aliases_and_compatible_versions() {
        let equivalent_pairs = [
            ("AWS", "Amazon Web Services"),
            ("GCP", "Google Cloud Platform"),
            ("JavaScript", "JS"),
            ("TypeScript", "TS"),
            ("PostgreSQL", "Postgres"),
            ("Kubernetes", "K8s"),
            ("Node.js", "NodeJS"),
            ("React.js", "React"),
            (".NET", "DotNet"),
            ("C#", "C Sharp"),
            ("C++", "CPP"),
            ("Go", "Golang"),
            ("Python 3.11", "Python"),
            ("Java v17", "Java 17"),
        ];
        for (job_skill, resume_skill) in equivalent_pairs {
            assert!(
                are_conservative_skill_equivalents(job_skill, resume_skill),
                "expected {job_skill:?} and {resume_skill:?} to be equivalent"
            );
        }
        assert_eq!(canonicalize_skill("Node.js 20"), "node");
    }

    #[test]
    fn every_configured_alias_is_equivalent_only_within_its_group() {
        for (group_index, group) in SKILL_EQUIVALENCE_GROUPS.iter().enumerate() {
            for left in *group {
                for right in *group {
                    assert!(
                        are_conservative_skill_equivalents(left, right),
                        "configured aliases {left:?} and {right:?} should match"
                    );
                }
                for other_group in SKILL_EQUIVALENCE_GROUPS.iter().skip(group_index + 1) {
                    for other in *other_group {
                        assert!(
                            !are_conservative_skill_equivalents(left, other),
                            "aliases from different groups {left:?} and {other:?} must not match"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn rejects_related_competing_and_differently_versioned_technologies() {
        let non_equivalent_pairs = [
            ("Kubernetes", "Docker"),
            ("PostgreSQL", "MySQL"),
            ("React", "Angular"),
            ("AWS", "Azure"),
            ("Terraform", "CloudFormation"),
            ("Django", "Flask"),
            ("Java", "JavaScript"),
            ("C", "C++"),
            ("Machine Learning", "Artificial Intelligence"),
            ("Python 2", "Python 3"),
            ("Java 8", "Java 17"),
        ];
        for (job_skill, resume_skill) in non_equivalent_pairs {
            assert!(
                !are_conservative_skill_equivalents(job_skill, resume_skill),
                "expected {job_skill:?} and {resume_skill:?} not to be equivalent"
            );
        }
    }
}
