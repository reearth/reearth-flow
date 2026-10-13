use std::net::IpAddr;

use once_cell::sync::Lazy;

use crate::error::EgressError;

pub const ALLOW_PRIVATE_NETWORK_ENV: &str = "FLOW_RUNTIME_HTTP_ALLOW_PRIVATE_NETWORK";

static ALLOW_PRIVATE_NETWORK: Lazy<bool> = Lazy::new(|| {
    std::env::var(ALLOW_PRIVATE_NETWORK_ENV)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
});

/// Which destinations outbound requests may reach.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EgressPolicy {
    /// Allow private, loopback and other internal addresses. Never-routable
    /// addresses stay blocked regardless.
    pub allow_private_network: bool,
}

impl EgressPolicy {
    /// Policy configured by `FLOW_RUNTIME_HTTP_ALLOW_PRIVATE_NETWORK`.
    pub fn from_env() -> Self {
        Self {
            allow_private_network: *ALLOW_PRIVATE_NETWORK,
        }
    }

    pub(crate) fn parse_url(&self, url: &str) -> Result<url::Url, EgressError> {
        let parsed: url::Url =
            url.parse()
                .map_err(|e: url::ParseError| EgressError::InvalidUrl {
                    url: url.to_string(),
                    message: e.to_string(),
                })?;
        self.check_url(&parsed)?;
        Ok(parsed)
    }

    pub(crate) fn check_url(&self, url: &url::Url) -> Result<(), EgressError> {
        match url.scheme() {
            "http" | "https" => {}
            other => return Err(EgressError::SchemeNotAllowed(other.to_string())),
        }
        let literal_ip = match url.host() {
            Some(url::Host::Ipv4(ip)) => IpAddr::V4(ip),
            Some(url::Host::Ipv6(ip)) => IpAddr::V6(ip),
            Some(url::Host::Domain(_)) => return Ok(()),
            None => {
                return Err(EgressError::InvalidUrl {
                    url: url.to_string(),
                    message: "URL has no host".to_string(),
                })
            }
        };
        if self.is_blocked(literal_ip) {
            return Err(self.blocked_error(&literal_ip.to_string(), &[literal_ip]));
        }
        Ok(())
    }

    /// True when the address must not be reached from a workflow.
    pub(crate) fn is_blocked(&self, ip: IpAddr) -> bool {
        is_blocked_ip(ip, self.allow_private_network)
    }

    /// Error for a host whose addresses are all blocked. The opt-out is
    /// mentioned only when it would actually let the request through.
    pub(crate) fn blocked_error(&self, host: &str, addrs: &[IpAddr]) -> EgressError {
        let mut message = format!(
            "Request to '{host}' is blocked: private, loopback and internal network \
             addresses are not allowed"
        );
        if !self.allow_private_network && addrs.iter().any(|ip| !is_blocked_ip(*ip, true)) {
            message.push_str(&format!(
                " (set {ALLOW_PRIVATE_NETWORK_ENV}=true to allow private network addresses)"
            ));
        }
        EgressError::Blocked(message)
    }
}

/// Addresses that are never valid HTTP targets (unspecified, broadcast,
/// multicast) stay blocked even when private networking is allowed; the
/// opt-out only covers private, loopback and other internal ranges.
fn is_blocked_ip(ip: IpAddr, allow_private: bool) -> bool {
    let ip = match ip {
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(mapped) => IpAddr::V4(mapped),
            None => IpAddr::V6(v6),
        },
        v4 @ IpAddr::V4(_) => v4,
    };
    is_never_routable(ip) || (!allow_private && is_private_or_internal(ip))
}

fn is_never_routable(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_unspecified() || v4.is_broadcast() || v4.is_multicast() || v4.octets()[0] == 0
            // 0.0.0.0/8 "this network"
        }
        IpAddr::V6(v6) => v6.is_unspecified() || v6.is_multicast(),
    }
}

fn is_private_or_internal(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local() // includes 169.254.169.254 cloud metadata
                || (o[0] == 100 && (o[1] & 0xc0) == 64) // 100.64.0.0/10 carrier-grade NAT
                || (o[0] == 192 && o[1] == 0 && o[2] == 0) // 192.0.0.0/24 IETF protocol assignments
        }
        IpAddr::V6(v6) => {
            let s = v6.segments();
            v6.is_loopback()
                || (s[0] & 0xfe00) == 0xfc00 // fc00::/7 unique-local
                || (s[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STRICT: EgressPolicy = EgressPolicy {
        allow_private_network: false,
    };
    const ALLOW_PRIVATE: EgressPolicy = EgressPolicy {
        allow_private_network: true,
    };

    #[test]
    fn test_blocks_non_http_schemes() {
        for url in [
            "file:///etc/passwd",
            "ftp://example.com/x",
            "gopher://example.com",
        ] {
            let err = STRICT.parse_url(url).unwrap_err();
            assert!(err.to_string().contains("not allowed"), "{url}: {err}");
            // The scheme restriction applies even when private addresses are allowed.
            assert!(ALLOW_PRIVATE.parse_url(url).is_err(), "{url}");
        }
    }

    #[test]
    fn test_blocks_literal_internal_ips() {
        for url in [
            "http://127.0.0.1/",
            "http://127.0.0.1:8080/admin",
            "http://0.0.0.0/",
            "http://10.0.0.5/",
            "http://172.16.0.1/",
            "http://192.168.1.1/",
            "http://169.254.169.254/latest/meta-data/",
            "http://100.64.0.1/",
            "http://[::1]/",
            "http://[fe80::1]/",
            "http://[fd00::1]/",
            "http://[::ffff:127.0.0.1]/",
        ] {
            assert!(STRICT.parse_url(url).is_err(), "should be blocked: {url}");
        }
    }

    #[test]
    fn test_allows_public_urls() {
        for url in [
            "https://example.com/api",
            "http://example.com:8080/x?y=z",
            "https://93.184.216.34/",
        ] {
            assert!(STRICT.parse_url(url).is_ok(), "should be allowed: {url}");
        }
    }

    #[test]
    fn test_allow_private_permits_internal_ips_but_not_schemes() {
        assert!(ALLOW_PRIVATE.parse_url("http://127.0.0.1/").is_ok());
        assert!(ALLOW_PRIVATE.parse_url("http://192.168.1.1/").is_ok());
        assert!(ALLOW_PRIVATE.parse_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn test_allow_private_still_blocks_never_routable_addresses() {
        for url in [
            "http://0.0.0.0/",
            "http://255.255.255.255/",
            "http://224.0.0.1/",
            "http://[::]/",
            "http://[ff02::1]/",
        ] {
            assert!(
                ALLOW_PRIVATE.parse_url(url).is_err(),
                "should be blocked even with private networking allowed: {url}"
            );
        }
    }

    #[test]
    fn test_blocked_message_mentions_opt_out_only_when_it_helps() {
        let err = STRICT.parse_url("http://127.0.0.1/").unwrap_err();
        assert!(err.to_string().contains(ALLOW_PRIVATE_NETWORK_ENV), "{err}");

        let err = STRICT.parse_url("http://0.0.0.0/").unwrap_err();
        assert!(
            !err.to_string().contains(ALLOW_PRIVATE_NETWORK_ENV),
            "{err}"
        );
    }

    #[test]
    fn test_blocked_ip_ranges() {
        let blocked = [
            "127.0.0.1",
            "10.1.2.3",
            "172.31.255.255",
            "192.168.0.1",
            "169.254.169.254",
            "0.0.0.0",
            "255.255.255.255",
            "224.0.0.1",
            "100.64.0.1",
            "100.127.255.255",
            "192.0.0.1",
            "::1",
            "::",
            "fd12:3456::1",
            "fe80::1",
            "ff02::1",
            "::ffff:10.0.0.1",
        ];
        for ip in blocked {
            assert!(
                is_blocked_ip(ip.parse().unwrap(), false),
                "should be blocked: {ip}"
            );
        }

        let allowed = [
            "93.184.216.34",
            "8.8.8.8",
            "100.63.255.255",
            "100.128.0.0",
            "2606:2800:220:1:248:1893:25c8:1946",
        ];
        for ip in allowed {
            assert!(
                !is_blocked_ip(ip.parse().unwrap(), false),
                "should be allowed: {ip}"
            );
        }
    }

    #[test]
    fn test_blocked_ip_split_with_allow_private() {
        // Private/internal: reachable with the opt-out.
        for ip in ["127.0.0.1", "10.1.2.3", "169.254.169.254", "::1", "fe80::1"] {
            assert!(
                !is_blocked_ip(ip.parse().unwrap(), true),
                "should be allowed with opt-out: {ip}"
            );
        }
        // Never routable: blocked in both modes.
        for ip in ["0.0.0.0", "255.255.255.255", "224.0.0.1", "::", "ff02::1"] {
            assert!(
                is_blocked_ip(ip.parse().unwrap(), true),
                "should be blocked with opt-out: {ip}"
            );
        }
    }
}
