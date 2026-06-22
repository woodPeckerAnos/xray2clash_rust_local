use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use vless_clash_dev::{
    build_profile, list_available_presets, parse_vless_link, profile_to_yaml, vless_to_proxy,
    ClashProxy, PresetSelection, ProfileMode,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputMode {
    Global,
    Rule,
    Whitelist,
    Both,
}

#[derive(Parser, Debug)]
#[command(
    name = "xray2clash",
    about = "Convert VLESS + REALITY share links to Clash Verge Dev YAML configs"
)]
struct Cli {
    /// Single vless:// share link
    #[arg(long, conflicts_with = "input_file")]
    input: Option<String>,

    /// Text file with one vless:// link per line
    #[arg(long, conflicts_with = "input")]
    input_file: Option<PathBuf>,

    /// Output YAML file path (single link input)
    #[arg(long, conflicts_with_all = ["output_dir", "stdout"])]
    output: Option<PathBuf>,

    /// Output directory for generated YAML files
    #[arg(long, conflicts_with_all = ["output", "stdout"])]
    output_dir: Option<PathBuf>,

    /// Write YAML to stdout
    #[arg(long, conflicts_with_all = ["output", "output_dir"])]
    stdout: bool,

    /// Profile mode to generate
    #[arg(long, value_enum, default_value_t = OutputMode::Rule)]
    mode: OutputMode,

    /// Whitelist preset to include (repeatable). Use with --mode whitelist
    #[arg(long = "preset", conflicts_with = "preset_all")]
    presets: Vec<String>,

    /// Use all built-in whitelist presets
    #[arg(long, conflicts_with = "presets")]
    preset_all: bool,

    /// Additional custom preset YAML file (repeatable)
    #[arg(long = "custom-preset")]
    custom_presets: Vec<PathBuf>,

    /// List built-in and user whitelist presets
    #[arg(long)]
    list_presets: bool,

    /// Rename proxy node shown in Clash Verge Dev
    #[arg(long, alias = "rename")]
    name: Option<String>,

    /// Output file basename without extension (used with --output-dir)
    #[arg(long)]
    output_name: Option<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_presets {
        return list_presets();
    }

    let links = read_links(&cli)?;

    if links.is_empty() {
        bail!("no vless:// links found in input");
    }

    let whitelist_selection = build_whitelist_selection(&cli)?;

    let proxies: Vec<ClashProxy> = links
        .iter()
        .enumerate()
        .map(|(index, link)| {
            let override_name = if links.len() == 1 {
                cli.name.clone()
            } else if let Some(name) = &cli.name {
                Some(format!("{name}-{}", index + 1))
            } else {
                None
            };

            let parsed = parse_vless_link(link)
                .with_context(|| format!("failed to parse link #{}", index + 1))?;
            Ok(vless_to_proxy(&parsed, override_name.as_deref()))
        })
        .collect::<Result<Vec<_>>>()?;

    let modes = output_modes(cli.mode);
    let whitelist_ref = whitelist_selection.as_ref();
    let multiple_outputs = modes.len() > 1;

    if cli.stdout {
        if modes.len() != 1 {
            bail!("--stdout only supports a single output mode; use --mode global, rule, or whitelist");
        }
        let yaml = render_yaml(&proxies, modes[0], whitelist_ref)?;
        print!("{yaml}");
        return Ok(());
    }

    if let Some(output) = &cli.output {
        if links.len() != 1 {
            bail!("--output only supports a single input link");
        }
        if modes.len() != 1 {
            bail!("--output only supports a single output mode; use --mode global, rule, or whitelist");
        }
        if let Some(parent) = output.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create output directory {}", parent.display())
                })?;
            }
        }
        write_yaml(output, &proxies, modes[0], whitelist_ref)?;
        return Ok(());
    }

    let output_dir = cli
        .output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("outputs"));

    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output directory {}", output_dir.display()))?;

    for profile_mode in modes {
        let suffix = match profile_mode {
            ProfileMode::Global => "global",
            ProfileMode::Rule => "rule",
            ProfileMode::Whitelist => "whitelist",
        };
        let filename = if multiple_outputs {
            format!(
                "{}-{}.yaml",
                output_basename(&cli, &proxies, links.len()),
                suffix
            )
        } else {
            format!("{}.yaml", output_basename(&cli, &proxies, links.len()))
        };

        let path = output_dir.join(filename);
        write_yaml(&path, &proxies, profile_mode, whitelist_ref)?;
        eprintln!("wrote {}", path.display());
    }

    Ok(())
}

fn list_presets() -> Result<()> {
    for preset in list_available_presets()? {
        let source = match preset.source {
            vless_clash_dev::PresetSource::Bundled => "bundled",
            vless_clash_dev::PresetSource::UserConfig => "user",
            vless_clash_dev::PresetSource::File(_) => "file",
        };
        let description = preset.description.unwrap_or_default();
        println!(
            "{}\t{}\t{}\t{} domains",
            preset.name, source, description, preset.domain_count
        );
    }
    Ok(())
}

fn build_whitelist_selection(cli: &Cli) -> Result<Option<PresetSelection>> {
    let needs_whitelist = matches!(cli.mode, OutputMode::Whitelist | OutputMode::Both);
    if !needs_whitelist {
        return Ok(None);
    }

    let has_explicit_selection =
        cli.preset_all || !cli.presets.is_empty() || !cli.custom_presets.is_empty();

    let selection = if has_explicit_selection {
        PresetSelection {
            preset_names: cli.presets.clone(),
            custom_preset_paths: cli.custom_presets.clone(),
            use_all_bundled: cli.preset_all,
        }
    } else {
        PresetSelection {
            use_all_bundled: true,
            ..Default::default()
        }
    };

    Ok(Some(selection))
}

fn read_links(cli: &Cli) -> Result<Vec<String>> {
    match (&cli.input, &cli.input_file) {
        (Some(input), None) => Ok(vec![input.clone()]),
        (None, Some(path)) => {
            let content = fs::read_to_string(path)
                .with_context(|| format!("failed to read input file {}", path.display()))?;
            Ok(content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_string)
                .collect())
        }
        (None, None) => {
            if cli.list_presets {
                return Ok(Vec::new());
            }
            bail!("one of --input or --input-file is required");
        }
        (Some(_), Some(_)) => unreachable!("clap should prevent both input sources"),
    }
}

fn output_modes(mode: OutputMode) -> Vec<ProfileMode> {
    match mode {
        OutputMode::Global => vec![ProfileMode::Global],
        OutputMode::Rule => vec![ProfileMode::Rule],
        OutputMode::Whitelist => vec![ProfileMode::Whitelist],
        OutputMode::Both => vec![
            ProfileMode::Global,
            ProfileMode::Rule,
            ProfileMode::Whitelist,
        ],
    }
}

fn render_yaml(
    proxies: &[ClashProxy],
    mode: ProfileMode,
    whitelist_selection: Option<&PresetSelection>,
) -> Result<String> {
    let profile = build_profile(proxies.to_vec(), mode, whitelist_selection)?;
    profile_to_yaml(&profile)
}

fn write_yaml(
    path: &Path,
    proxies: &[ClashProxy],
    mode: ProfileMode,
    whitelist_selection: Option<&PresetSelection>,
) -> Result<()> {
    let yaml = render_yaml(proxies, mode, whitelist_selection)?;
    fs::write(path, yaml).with_context(|| format!("failed to write {}", path.display()))
}

fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect();

    if sanitized.is_empty() {
        "profile".to_string()
    } else {
        sanitized
    }
}

fn output_basename(cli: &Cli, proxies: &[ClashProxy], link_count: usize) -> String {
    if let Some(name) = &cli.output_name {
        return sanitize_filename(name);
    }

    if link_count == 1 {
        if let Some(name) = cli.name.as_deref() {
            return sanitize_filename(name);
        }
        if let Some(proxy) = proxies.first() {
            return sanitize_filename(&proxy.name);
        }
    }

    sanitize_filename("profile")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_filename_replaces_spaces() {
        assert_eq!(sanitize_filename("Example US"), "Example-US");
    }

    #[test]
    fn output_basename_prefers_output_name() {
        let cli = Cli::try_parse_from([
            "xray2clash",
            "--input",
            "vless://uuid@example.com:443?security=reality&sni=www.example.com&pbk=KEY#Node",
            "--name",
            "Proxy-Node",
            "--output-name",
            "My-Config",
        ])
        .expect("cli");
        let parsed = parse_vless_link(
            "vless://uuid@example.com:443?security=reality&sni=www.example.com&pbk=KEY#Node",
        )
        .expect("parse");
        let proxies = vec![vless_to_proxy(&parsed, cli.name.as_deref())];

        assert_eq!(output_basename(&cli, &proxies, 1), "My-Config");
    }

    #[test]
    fn rename_alias_maps_to_name() {
        let cli = Cli::try_parse_from([
            "xray2clash",
            "--input",
            "vless://uuid@example.com:443?security=reality&sni=www.example.com&pbk=KEY",
            "--rename",
            "US-Node",
        ])
        .expect("cli");

        assert_eq!(cli.name.as_deref(), Some("US-Node"));
    }

    #[test]
    fn whitelist_defaults_to_all_presets() {
        let cli = Cli::try_parse_from([
            "xray2clash",
            "--input",
            "vless://uuid@example.com:443?security=reality&sni=www.example.com&pbk=KEY",
            "--mode",
            "whitelist",
        ])
        .expect("cli");

        let selection = build_whitelist_selection(&cli)
            .expect("selection")
            .expect("whitelist");
        assert!(selection.use_all_bundled);
        assert!(selection.preset_names.is_empty());
    }
}
