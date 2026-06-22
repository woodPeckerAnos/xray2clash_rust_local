use crate::preset::resolve_preset_domains;
use crate::preset::PresetSelection;

/// Private network and localhost always go direct.
pub fn direct_bypass_rules() -> Vec<String> {
    vec![
        "DOMAIN-SUFFIX,local,DIRECT".to_string(),
        "IP-CIDR,127.0.0.0/8,DIRECT,no-resolve".to_string(),
        "IP-CIDR,10.0.0.0/8,DIRECT,no-resolve".to_string(),
        "IP-CIDR,172.16.0.0/12,DIRECT,no-resolve".to_string(),
        "IP-CIDR,192.168.0.0/16,DIRECT,no-resolve".to_string(),
        "IP-CIDR,169.254.0.0/16,DIRECT,no-resolve".to_string(),
    ]
}

pub fn domains_to_proxy_rules(domains: &[String]) -> Vec<String> {
    domains
        .iter()
        .map(|domain| format!("DOMAIN-SUFFIX,{domain},PROXY"))
        .collect()
}

pub fn whitelist_rules_from_domains(domains: &[String]) -> Vec<String> {
    let mut rules = direct_bypass_rules();
    rules.extend(domains_to_proxy_rules(domains));
    rules.push("MATCH,DIRECT".to_string());
    rules
}

pub fn whitelist_rules(selection: &PresetSelection) -> anyhow::Result<Vec<String>> {
    let domains = resolve_preset_domains(selection)?;
    Ok(whitelist_rules_from_domains(&domains))
}

pub fn default_whitelist_selection() -> PresetSelection {
    PresetSelection {
        use_all_bundled: true,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::PresetSelection;

    #[test]
    fn whitelist_uses_domain_suffix_only() {
        let rules = whitelist_rules(&default_whitelist_selection()).expect("rules");
        assert!(rules.iter().all(|rule| !rule.starts_with("GEOSITE,")));
        assert_eq!(rules.last().map(String::as_str), Some("MATCH,DIRECT"));
    }

    #[test]
    fn whitelist_includes_githubusercontent_and_android() {
        let rules = whitelist_rules(&PresetSelection {
            preset_names: vec!["github".to_string(), "android".to_string()],
            ..Default::default()
        })
        .expect("rules");

        assert!(rules
            .iter()
            .any(|rule| rule.contains("githubusercontent.com")));
        assert!(rules.iter().any(|rule| rule.contains("developer.android.com")));
        assert!(rules.iter().any(|rule| rule.contains("maven.google.com")));
        assert!(rules.iter().any(|rule| rule.contains("gradle.org")));
    }

    #[test]
    fn bundled_presets_exclude_apple() {
        let rules = whitelist_rules(&default_whitelist_selection()).expect("rules");
        assert!(!rules.iter().any(|rule| rule.contains("apple.com")));
        assert!(!rules.iter().any(|rule| rule.contains("icloud.com")));
    }
}
