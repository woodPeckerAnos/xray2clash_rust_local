use percent_encoding::percent_decode_str;
use thiserror::Error;
use url::Url;

use crate::models::VlessLink;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("input must start with vless://")]
    InvalidScheme,
    #[error("failed to parse URL: {0}")]
    UrlParse(String),
    #[error("missing user info (UUID) in link")]
    MissingUuid,
    #[error("missing host in link")]
    MissingHost,
    #[error("missing port in link")]
    MissingPort,
    #[error("invalid port: {0}")]
    InvalidPort(String),
    #[error("only VLESS + REALITY is supported, got security={0}")]
    UnsupportedSecurity(String),
    #[error("missing required query parameter: {0}")]
    MissingParam(&'static str),
}

fn decode_component(value: &str) -> String {
    percent_decode_str(value)
        .decode_utf8_lossy()
        .into_owned()
}

fn query_param<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    pairs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

pub fn parse_vless_link(input: &str) -> Result<VlessLink, ParseError> {
    let trimmed = input.trim();
    if !trimmed.starts_with("vless://") {
        return Err(ParseError::InvalidScheme);
    }

    let url = Url::parse(trimmed).map_err(|e| ParseError::UrlParse(e.to_string()))?;

    let uuid = url
        .username()
        .trim()
        .to_string();
    if uuid.is_empty() {
        return Err(ParseError::MissingUuid);
    }

    let server = url
        .host_str()
        .ok_or(ParseError::MissingHost)?
        .to_string();

    let port = url.port_or_known_default().ok_or(ParseError::MissingPort)?;

    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    let security = query_param(&pairs, "security")
        .ok_or(ParseError::MissingParam("security"))?
        .to_string();
    if security != "reality" {
        return Err(ParseError::UnsupportedSecurity(security));
    }

    let sni = query_param(&pairs, "sni")
        .ok_or(ParseError::MissingParam("sni"))?
        .to_string();

    let public_key = query_param(&pairs, "pbk")
        .ok_or(ParseError::MissingParam("pbk"))?
        .to_string();

    let network = query_param(&pairs, "type")
        .unwrap_or("tcp")
        .to_string();

    let fingerprint = query_param(&pairs, "fp")
        .unwrap_or("chrome")
        .to_string();

    let short_id = query_param(&pairs, "sid")
        .filter(|sid| !sid.is_empty())
        .map(|sid| sid.to_string());

    let flow = query_param(&pairs, "flow")
        .filter(|flow| !flow.is_empty())
        .map(|flow| flow.to_string());

    let name = url
        .fragment()
        .filter(|fragment| !fragment.is_empty())
        .map(decode_component);

    Ok(VlessLink {
        uuid,
        server,
        port,
        name,
        network,
        security,
        sni,
        fingerprint,
        public_key,
        short_id,
        flow,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_LINK: &str = "vless://4d6e0338-f67a-4187-bca3-902e232466bc@example.com:443?type=tcp&security=reality&sni=www.example.com&fp=chrome&pbk=PUBLIC_KEY&sid=SHORT_ID&flow=xtls-rprx-vision#Example-US";

    #[test]
    fn parses_standard_reality_link() {
        let link = parse_vless_link(SAMPLE_LINK).expect("parse link");

        assert_eq!(link.uuid, "4d6e0338-f67a-4187-bca3-902e232466bc");
        assert_eq!(link.server, "example.com");
        assert_eq!(link.port, 443);
        assert_eq!(link.network, "tcp");
        assert_eq!(link.security, "reality");
        assert_eq!(link.sni, "www.example.com");
        assert_eq!(link.fingerprint, "chrome");
        assert_eq!(link.public_key, "PUBLIC_KEY");
        assert_eq!(link.short_id.as_deref(), Some("SHORT_ID"));
        assert_eq!(link.flow.as_deref(), Some("xtls-rprx-vision"));
        assert_eq!(link.name.as_deref(), Some("Example-US"));
    }

    #[test]
    fn defaults_fingerprint_to_chrome() {
        let link = parse_vless_link(
            "vless://uuid@example.com:443?security=reality&sni=www.example.com&pbk=KEY",
        )
        .expect("parse link");

        assert_eq!(link.fingerprint, "chrome");
    }

    #[test]
    fn omits_empty_short_id() {
        let link = parse_vless_link(
            "vless://uuid@example.com:443?security=reality&sni=www.example.com&pbk=KEY&sid=",
        )
        .expect("parse link");

        assert!(link.short_id.is_none());
    }

    #[test]
    fn decodes_fragment_name() {
        let link = parse_vless_link(
            "vless://uuid@example.com:443?security=reality&sni=www.example.com&pbk=KEY#%E4%B8%AD%E6%96%87%20Node",
        )
        .expect("parse link");

        assert_eq!(link.name.as_deref(), Some("中文 Node"));
    }

    #[test]
    fn rejects_non_reality_security() {
        let err = parse_vless_link(
            "vless://uuid@example.com:443?security=tls&sni=www.example.com&pbk=KEY",
        )
        .unwrap_err();

        assert_eq!(
            err,
            ParseError::UnsupportedSecurity("tls".to_string())
        );
    }

    #[test]
    fn rejects_invalid_scheme() {
        let err = parse_vless_link("vmess://uuid@example.com:443").unwrap_err();
        assert_eq!(err, ParseError::InvalidScheme);
    }
}
