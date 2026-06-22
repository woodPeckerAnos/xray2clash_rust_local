use vless_clash_dev::{
    build_profile, parse_vless_link, profile_to_yaml, vless_to_proxy, PresetSelection, ProfileMode,
};

const SAMPLE_LINK: &str = "vless://4d6e0338-f67a-4187-bca3-902e232466bc@example.com:443?type=tcp&security=reality&sni=www.example.com&fp=chrome&pbk=PUBLIC_KEY&sid=SHORT_ID&flow=xtls-rprx-vision#Example-US";

#[test]
fn integration_generates_importable_yaml() {
    let link = parse_vless_link(SAMPLE_LINK).expect("parse");
    let proxy = vless_to_proxy(&link, None);
    let profile = build_profile(vec![proxy], ProfileMode::Rule, None).expect("profile");
    let yaml = profile_to_yaml(&profile).expect("yaml");

    assert!(yaml.contains("mixed-port: 7890"));
    assert!(yaml.contains("type: vless"));
    assert!(yaml.contains("client-fingerprint: chrome"));
    assert!(yaml.contains("public-key: PUBLIC_KEY"));
    assert!(yaml.contains("short-id: SHORT_ID"));
    assert!(yaml.contains("flow: xtls-rprx-vision"));
    assert!(yaml.contains("- name: PROXY"));
    assert!(yaml.contains("- Example-US"));
}

#[test]
fn integration_whitelist_mode_uses_direct_fallback() {
    let link = parse_vless_link(SAMPLE_LINK).expect("parse");
    let proxy = vless_to_proxy(&link, None);
    let profile = build_profile(
        vec![proxy],
        ProfileMode::Whitelist,
        Some(&PresetSelection {
            preset_names: vec!["github".to_string(), "android".to_string()],
            ..Default::default()
        }),
    )
    .expect("profile");
    let yaml = profile_to_yaml(&profile).expect("yaml");

    assert!(yaml.contains("DOMAIN-SUFFIX,githubusercontent.com,PROXY"));
    assert!(yaml.contains("DOMAIN-SUFFIX,developer.android.com,PROXY"));
    assert!(yaml.contains("MATCH,DIRECT"));
    assert!(!yaml.contains("GEOSITE,"));
}

#[test]
fn integration_supports_multiple_links() {
    let links = [
        "vless://uuid-a@a.example.com:443?security=reality&sni=www.example.com&pbk=KEY#Node-A",
        "vless://uuid-b@b.example.com:443?security=reality&sni=www.example.com&pbk=KEY#Node-B",
    ];

    let proxies: Vec<_> = links
        .iter()
        .map(|link| vless_to_proxy(&parse_vless_link(link).expect("parse"), None))
        .collect();

    let profile = build_profile(proxies, ProfileMode::Rule, None).expect("profile");
    let yaml = profile_to_yaml(&profile).expect("yaml");

    assert!(yaml.contains("- Node-A"));
    assert!(yaml.contains("- Node-B"));
    assert!(yaml.contains("proxies:\n  - Node-A\n  - Node-B") || yaml.contains("- Node-A"));
}
