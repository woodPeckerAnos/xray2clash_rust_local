use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Preset {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetSummary {
    pub name: String,
    pub description: Option<String>,
    pub source: PresetSource,
    pub domain_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresetSource {
    Bundled,
    UserConfig,
    File(PathBuf),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PresetError {
    #[error("preset not found: {0}")]
    NotFound(String),
    #[error("invalid domain in preset `{preset}`: {domain}")]
    InvalidDomain { preset: String, domain: String },
}

#[derive(Debug, Clone, Default)]
pub struct PresetSelection {
    pub preset_names: Vec<String>,
    pub custom_preset_paths: Vec<PathBuf>,
    pub use_all_bundled: bool,
}

struct BundledPreset {
    name: &'static str,
    content: &'static str,
}

const BUNDLED_PRESETS: &[BundledPreset] = &[
    BundledPreset {
        name: "google",
        content: include_str!("../presets/google.yaml"),
    },
    BundledPreset {
        name: "github",
        content: include_str!("../presets/github.yaml"),
    },
    BundledPreset {
        name: "android",
        content: include_str!("../presets/android.yaml"),
    },
    BundledPreset {
        name: "ai",
        content: include_str!("../presets/ai.yaml"),
    },
    BundledPreset {
        name: "social",
        content: include_str!("../presets/social.yaml"),
    },
    BundledPreset {
        name: "devtools",
        content: include_str!("../presets/devtools.yaml"),
    },
    BundledPreset {
        name: "productivity",
        content: include_str!("../presets/productivity.yaml"),
    },
    BundledPreset {
        name: "media",
        content: include_str!("../presets/media.yaml"),
    },
];

pub fn bundled_preset_names() -> Vec<&'static str> {
    BUNDLED_PRESETS.iter().map(|preset| preset.name).collect()
}

pub fn user_preset_dir() -> PathBuf {
    dirs_config_home().join("xray2clash").join("presets")
}

pub fn list_available_presets() -> Result<Vec<PresetSummary>> {
    let mut summaries = Vec::new();

    for bundled in BUNDLED_PRESETS {
        let preset = parse_preset_yaml(bundled.content, bundled.name)?;
        summaries.push(PresetSummary {
            name: preset.name,
            description: preset.description,
            source: PresetSource::Bundled,
            domain_count: preset.domains.len(),
        });
    }

    let user_dir = user_preset_dir();
    if user_dir.is_dir() {
        for entry in fs::read_dir(&user_dir).with_context(|| {
            format!("failed to read user preset directory {}", user_dir.display())
        })? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
                continue;
            }
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read preset {}", path.display()))?;
            let preset = parse_preset_yaml(&content, path.to_string_lossy().as_ref())?;
            summaries.push(PresetSummary {
                name: preset.name.clone(),
                description: preset.description,
                source: PresetSource::UserConfig,
                domain_count: preset.domains.len(),
            });
        }
    }

    summaries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(summaries)
}

pub fn resolve_preset_domains(selection: &PresetSelection) -> Result<Vec<String>> {
    let presets = load_selected_presets(selection)?;
    if presets.is_empty() {
        bail!("no whitelist presets selected; use --preset-all, --preset <name>, or --custom-preset <file>");
    }

    let mut domains = BTreeSet::new();
    for preset in presets {
        for domain in normalize_domains(&preset)? {
            domains.insert(domain);
        }
    }

    Ok(domains.into_iter().collect())
}

fn load_selected_presets(selection: &PresetSelection) -> Result<Vec<Preset>> {
    let mut presets = Vec::new();
    let bundled = bundled_preset_map()?;
    let user_presets = load_user_presets()?;

    if selection.use_all_bundled {
        for name in bundled_preset_names() {
            presets.push(
                bundled
                    .get(name)
                    .expect("bundled preset map should contain all bundled names")
                    .clone(),
            );
        }
    }

    for name in &selection.preset_names {
        if let Some(preset) = bundled.get(name) {
            presets.push(preset.clone());
            continue;
        }
        if let Some(preset) = user_presets.get(name) {
            presets.push(preset.clone());
            continue;
        }
        return Err(PresetError::NotFound(name.clone()).into());
    }

    for path in &selection.custom_preset_paths {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read custom preset {}", path.display()))?;
        let preset = parse_preset_yaml(&content, path.to_string_lossy().as_ref())?;
        presets.push(preset);
    }

    Ok(presets)
}

fn bundled_preset_map() -> Result<HashMap<String, Preset>> {
    let mut map = HashMap::new();
    for bundled in BUNDLED_PRESETS {
        let preset = parse_preset_yaml(bundled.content, bundled.name)?;
        map.insert(preset.name.clone(), preset);
    }
    Ok(map)
}

fn load_user_presets() -> Result<HashMap<String, Preset>> {
    let mut map = HashMap::new();
    let user_dir = user_preset_dir();
    if !user_dir.is_dir() {
        return Ok(map);
    }

    for entry in fs::read_dir(&user_dir).with_context(|| {
        format!("failed to read user preset directory {}", user_dir.display())
    })? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read preset {}", path.display()))?;
        let preset = parse_preset_yaml(&content, path.to_string_lossy().as_ref())?;
        map.insert(preset.name.clone(), preset);
    }

    Ok(map)
}

fn parse_preset_yaml(content: &str, source_label: &str) -> Result<Preset> {
    let preset: Preset = serde_yaml::from_str(content)
        .with_context(|| format!("failed to parse preset `{source_label}`"))?;

    if preset.name.trim().is_empty() {
        bail!("preset `{source_label}` is missing `name`");
    }
    if preset.domains.is_empty() {
        bail!("preset `{}` must contain at least one domain", preset.name);
    }

    Ok(preset)
}

fn normalize_domains(preset: &Preset) -> Result<Vec<String>> {
    preset
        .domains
        .iter()
        .map(|domain| normalize_domain(&preset.name, domain))
        .collect()
}

fn normalize_domain(preset_name: &str, domain: &str) -> Result<String> {
    let normalized = domain.trim().trim_start_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(PresetError::InvalidDomain {
            preset: preset_name.to_string(),
            domain: domain.to_string(),
        }
        .into());
    }
    if normalized.contains('/') || normalized.contains(':') || normalized.contains(' ') {
        return Err(PresetError::InvalidDomain {
            preset: preset_name.to_string(),
            domain: domain.to_string(),
        }
        .into());
    }
    Ok(normalized)
}

fn dirs_config_home() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }

    dirs_home().join(".config")
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_presets_parse() {
        for name in bundled_preset_names() {
            let map = bundled_preset_map().expect("bundled presets");
            assert!(map.contains_key(name));
        }
    }

    #[test]
    fn resolve_all_bundled_presets() {
        let domains = resolve_preset_domains(&PresetSelection {
            use_all_bundled: true,
            ..Default::default()
        })
        .expect("domains");

        assert!(domains.iter().any(|domain| domain == "githubusercontent.com"));
        assert!(domains.iter().any(|domain| domain == "developer.android.com"));
    }

    #[test]
    fn resolve_selected_subset() {
        let domains = resolve_preset_domains(&PresetSelection {
            preset_names: vec!["github".to_string(), "android".to_string()],
            ..Default::default()
        })
        .expect("domains");

        assert!(domains.iter().any(|domain| domain == "github.com"));
        assert!(domains.iter().any(|domain| domain == "gradle.org"));
        assert!(!domains.iter().any(|domain| domain == "openai.com"));
    }

    #[test]
    fn resolve_custom_preset_from_yaml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("mine.yaml");
        fs::write(
            &path,
            "name: mine\ndescription: test\ndomains:\n  - cursor.com\n",
        )
        .expect("write");

        let domains = resolve_preset_domains(&PresetSelection {
            custom_preset_paths: vec![path],
            ..Default::default()
        })
        .expect("domains");

        assert_eq!(domains, vec!["cursor.com".to_string()]);
    }
}
