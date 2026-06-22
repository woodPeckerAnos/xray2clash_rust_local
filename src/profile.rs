use anyhow::{Context, Result};
use serde_yaml;

use crate::models::{ClashProfile, ClashProxy, ProfileMode, ProxyGroup};
use crate::preset::PresetSelection;
use crate::whitelist::{default_whitelist_selection, whitelist_rules};

pub fn build_profile(
    proxies: Vec<ClashProxy>,
    mode: ProfileMode,
    whitelist_selection: Option<&PresetSelection>,
) -> Result<ClashProfile> {
    let proxy_names: Vec<String> = proxies.iter().map(|proxy| proxy.name.clone()).collect();

    let proxy_groups = vec![ProxyGroup {
        name: "PROXY".to_string(),
        group_type: "select".to_string(),
        proxies: proxy_names,
    }];

    let rules = match mode {
        ProfileMode::Global => vec!["MATCH,PROXY".to_string()],
        ProfileMode::Rule => vec![
            "DOMAIN-SUFFIX,local,DIRECT".to_string(),
            "IP-CIDR,127.0.0.0/8,DIRECT,no-resolve".to_string(),
            "IP-CIDR,10.0.0.0/8,DIRECT,no-resolve".to_string(),
            "IP-CIDR,172.16.0.0/12,DIRECT,no-resolve".to_string(),
            "IP-CIDR,192.168.0.0/16,DIRECT,no-resolve".to_string(),
            "IP-CIDR,169.254.0.0/16,DIRECT,no-resolve".to_string(),
            "GEOIP,CN,DIRECT".to_string(),
            "GEOSITE,CN,DIRECT".to_string(),
            "MATCH,PROXY".to_string(),
        ],
        ProfileMode::Whitelist => {
            let default_selection = default_whitelist_selection();
            let selection = whitelist_selection.unwrap_or(&default_selection);
            whitelist_rules(selection)?
        }
    };

    Ok(ClashProfile {
        mixed_port: 7890,
        allow_lan: false,
        mode: match mode {
            ProfileMode::Global => "global",
            ProfileMode::Rule | ProfileMode::Whitelist => "rule",
        }
        .to_string(),
        log_level: "info".to_string(),
        external_controller: "127.0.0.1:9090".to_string(),
        proxies,
        proxy_groups,
        rules,
    })
}

pub fn profile_to_yaml(profile: &ClashProfile) -> Result<String> {
    serde_yaml::to_string(profile).context("failed to serialize profile to YAML")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::vless_to_proxy;
    use crate::parser::parse_vless_link;
    use crate::preset::PresetSelection;

    fn sample_proxy(name: &str) -> ClashProxy {
        let link = parse_vless_link(&format!(
            "vless://uuid@example.com:443?security=reality&sni=www.example.com&pbk=KEY#{name}"
        ))
        .expect("parse");
        vless_to_proxy(&link, None)
    }

    #[test]
    fn rule_profile_contains_proxy_group() {
        let profile =
            build_profile(vec![sample_proxy("Node-A")], ProfileMode::Rule, None).expect("profile");

        assert_eq!(profile.mode, "rule");
        assert_eq!(profile.proxies.len(), 1);
        assert_eq!(profile.proxy_groups[0].proxies, vec!["Node-A".to_string()]);
        assert!(profile.rules.iter().any(|rule| rule == "GEOIP,CN,DIRECT"));
        assert_eq!(profile.rules.last().map(String::as_str), Some("MATCH,PROXY"));
    }

    #[test]
    fn global_profile_uses_match_rule() {
        let profile =
            build_profile(vec![sample_proxy("Node-A")], ProfileMode::Global, None).expect("profile");

        assert_eq!(profile.mode, "global");
        assert_eq!(profile.rules, vec!["MATCH,PROXY".to_string()]);
    }

    #[test]
    fn whitelist_profile_ends_with_direct_match() {
        let profile = build_profile(
            vec![sample_proxy("Node-A")],
            ProfileMode::Whitelist,
            Some(&PresetSelection {
                preset_names: vec!["github".to_string()],
                ..Default::default()
            }),
        )
        .expect("profile");

        assert_eq!(profile.mode, "rule");
        assert!(profile
            .rules
            .iter()
            .any(|rule| rule.contains("githubusercontent.com")));
        assert!(profile.rules.iter().all(|rule| !rule.starts_with("GEOSITE,")));
        assert_eq!(profile.rules.last().map(String::as_str), Some("MATCH,DIRECT"));
    }

    #[test]
    fn serializes_without_short_id_when_missing() {
        let link = parse_vless_link(
            "vless://uuid@example.com:443?security=reality&sni=www.example.com&pbk=KEY",
        )
        .expect("parse");
        let profile = build_profile(vec![vless_to_proxy(&link, None)], ProfileMode::Rule, None)
            .expect("profile");
        let yaml = profile_to_yaml(&profile).expect("yaml");

        assert!(yaml.contains("public-key: KEY"));
        assert!(!yaml.contains("short-id"));
    }
}
