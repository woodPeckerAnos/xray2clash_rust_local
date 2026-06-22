use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VlessLink {
    pub uuid: String,
    pub server: String,
    pub port: u16,
    pub name: Option<String>,
    pub network: String,
    pub security: String,
    pub sni: String,
    pub fingerprint: String,
    pub public_key: String,
    pub short_id: Option<String>,
    pub flow: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RealityOpts {
    #[serde(rename = "public-key")]
    pub public_key: String,
    #[serde(rename = "short-id", skip_serializing_if = "Option::is_none")]
    pub short_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ClashProxy {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub server: String,
    pub port: u16,
    pub uuid: String,
    pub network: String,
    pub tls: bool,
    pub udp: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
    #[serde(rename = "client-fingerprint")]
    pub client_fingerprint: String,
    pub servername: String,
    #[serde(rename = "reality-opts")]
    pub reality_opts: RealityOpts,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub proxies: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileMode {
    Global,
    Rule,
    Whitelist,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ClashProfile {
    #[serde(rename = "mixed-port")]
    pub mixed_port: u16,
    #[serde(rename = "allow-lan")]
    pub allow_lan: bool,
    pub mode: String,
    #[serde(rename = "log-level")]
    pub log_level: String,
    #[serde(rename = "external-controller")]
    pub external_controller: String,
    pub proxies: Vec<ClashProxy>,
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<String>,
}
