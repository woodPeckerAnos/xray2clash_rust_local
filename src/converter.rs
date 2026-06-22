use crate::models::{ClashProxy, RealityOpts, VlessLink};

pub fn vless_to_proxy(link: &VlessLink, name_override: Option<&str>) -> ClashProxy {
    let name = name_override
        .map(str::to_string)
        .or_else(|| link.name.clone())
        .unwrap_or_else(|| format!("{}:{}", link.server, link.port));

    ClashProxy {
        name,
        proxy_type: "vless".to_string(),
        server: link.server.clone(),
        port: link.port,
        uuid: link.uuid.clone(),
        network: link.network.clone(),
        tls: true,
        udp: true,
        flow: link.flow.clone(),
        client_fingerprint: link.fingerprint.clone(),
        servername: link.sni.clone(),
        reality_opts: RealityOpts {
            public_key: link.public_key.clone(),
            short_id: link.short_id.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_vless_link;

    #[test]
    fn converts_to_clash_proxy() {
        let link = parse_vless_link(
            "vless://uuid@example.com:443?security=reality&sni=www.example.com&pbk=KEY&sid=SID&flow=xtls-rprx-vision#Node",
        )
        .expect("parse");

        let proxy = vless_to_proxy(&link, None);

        assert_eq!(proxy.name, "Node");
        assert_eq!(proxy.proxy_type, "vless");
        assert_eq!(proxy.server, "example.com");
        assert_eq!(proxy.port, 443);
        assert_eq!(proxy.uuid, "uuid");
        assert_eq!(proxy.network, "tcp");
        assert!(proxy.tls);
        assert!(proxy.udp);
        assert_eq!(proxy.flow.as_deref(), Some("xtls-rprx-vision"));
        assert_eq!(proxy.client_fingerprint, "chrome");
        assert_eq!(proxy.servername, "www.example.com");
        assert_eq!(proxy.reality_opts.public_key, "KEY");
        assert_eq!(proxy.reality_opts.short_id.as_deref(), Some("SID"));
    }

    #[test]
    fn name_override_takes_precedence() {
        let link = parse_vless_link(
            "vless://uuid@example.com:443?security=reality&sni=www.example.com&pbk=KEY#Original",
        )
        .expect("parse");

        let proxy = vless_to_proxy(&link, Some("Override"));
        assert_eq!(proxy.name, "Override");
    }
}
