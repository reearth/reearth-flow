//! Network egress control for HTTP Caller.
//!
//! The engine executes user-authored workflows server-side, so an unrestricted
//! HTTP client is a server-side request forgery primitive: cloud metadata
//! endpoints, loopback services and private-network hosts would all be
//! reachable from URLs built out of feature data. This module is the network
//! counterpart of the filesystem sandbox that gates sink writes.
//!
//! Three layers, so a blocked address cannot be reached through any route:
//! 1. [`validate_url`] — checked per feature before sending: scheme must be
//!    http/https, and a literal-IP host must not be a blocked address.
//! 2. [`EgressGuardedDnsResolver`] — installed on the shared client: every
//!    hostname resolution (initial request, every redirect hop, every retry)
//!    drops blocked addresses, which also closes DNS-rebinding gaps because
//!    filtering happens at connection time.
//! 3. [`redirect_policy`] — literal-IP redirect targets never hit DNS, so each
//!    hop is checked here as well.
//!
//! Self-hosted deployments that legitimately call services on private
//! addresses can opt out with `FLOW_RUNTIME_HTTP_ALLOW_PRIVATE_NETWORK=true`
//! (the scheme restriction always applies).

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};

use once_cell::sync::Lazy;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use reqwest::redirect;

use super::errors::{HttpProcessorError, Result};

static ALLOW_PRIVATE_NETWORK: Lazy<bool> = Lazy::new(|| {
    std::env::var("FLOW_RUNTIME_HTTP_ALLOW_PRIVATE_NETWORK")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
});

pub(crate) fn allow_private_network() -> bool {
    *ALLOW_PRIVATE_NETWORK
}

/// Validate a fully-evaluated request URL before it is sent.
pub(crate) fn validate_url(url_str: &str) -> Result<()> {
    let url: url::Url = url_str
        .parse()
        .map_err(|e| HttpProcessorError::Request(format!("Invalid URL '{url_str}': {e}")))?;
    validate_parsed_url(&url, allow_private_network())
}

fn validate_parsed_url(url: &url::Url, allow_private: bool) -> Result<()> {
    match url.scheme() {
        "http" | "https" => {}
        other => {
            return Err(HttpProcessorError::Request(format!(
                "URL scheme '{other}' is not allowed; only http and https are supported"
            )));
        }
    }
    let Some(host) = url.host() else {
        return Err(HttpProcessorError::Request(format!(
            "URL '{url}' has no host"
        )));
    };
    if allow_private {
        return Ok(());
    }
    let literal_ip = match host {
        url::Host::Ipv4(ip) => Some(IpAddr::V4(ip)),
        url::Host::Ipv6(ip) => Some(IpAddr::V6(ip)),
        url::Host::Domain(_) => None,
    };
    if let Some(ip) = literal_ip {
        if is_blocked_ip(ip) {
            return Err(HttpProcessorError::Request(blocked_message(
                &ip.to_string(),
            )));
        }
    }
    Ok(())
}

fn blocked_message(host: &str) -> String {
    format!(
        "Request to '{host}' is blocked: private, loopback and internal network \
         addresses are not allowed"
    )
}

/// True when the address must not be reached from a workflow.
pub(crate) fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_ipv4(v4),
        IpAddr::V6(v6) => {
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return is_blocked_ipv4(mapped);
            }
            is_blocked_ipv6(v6)
        }
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local() // includes 169.254.169.254 cloud metadata
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_multicast()
        || o[0] == 0 // 0.0.0.0/8 "this network"
        || (o[0] == 100 && (o[1] & 0xc0) == 64) // 100.64.0.0/10 carrier-grade NAT
        || (o[0] == 192 && o[1] == 0 && o[2] == 0) // 192.0.0.0/24 IETF protocol assignments
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (s[0] & 0xfe00) == 0xfc00 // fc00::/7 unique-local
        || (s[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
}

/// DNS resolver that drops blocked addresses from every resolution, so a
/// hostname pointing (or re-pointing) at an internal address can never be
/// connected to.
#[derive(Debug, Clone, Default)]
pub(crate) struct EgressGuardedDnsResolver;

impl Resolve for EgressGuardedDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            let addrs = tokio::task::spawn_blocking({
                let host = host.clone();
                move || (host.as_str(), 0).to_socket_addrs().map(Vec::from_iter)
            })
            .await
            .map_err(io_error)?
            .map_err(io_error)?;

            let allow = allow_private_network();
            let allowed: Vec<SocketAddr> = addrs
                .into_iter()
                .filter(|addr| allow || !is_blocked_ip(addr.ip()))
                .collect();

            if allowed.is_empty() {
                return Err(io_error(blocked_message(&host)));
            }
            Ok(Box::new(allowed.into_iter()) as Addrs)
        })
    }
}

fn io_error(e: impl ToString) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::other(e.to_string()))
}

/// Redirect policy enforcing the hop limit plus per-hop scheme and
/// literal-IP checks. Hostname hops are covered by the guarded resolver.
pub(crate) fn redirect_policy(max_redirects: usize) -> redirect::Policy {
    redirect::Policy::custom(move |attempt| {
        if attempt.previous().len() > max_redirects {
            return attempt.error(format!("too many redirects (limit: {max_redirects})"));
        }
        if let Err(e) = validate_parsed_url(attempt.url(), allow_private_network()) {
            let message = e.to_string();
            return attempt.error(message);
        }
        attempt.follow()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(url: &str) -> url::Url {
        url.parse().unwrap()
    }

    #[test]
    fn test_blocks_non_http_schemes() {
        for url in [
            "file:///etc/passwd",
            "ftp://example.com/x",
            "gopher://example.com",
        ] {
            let err = validate_parsed_url(&parse(url), false).unwrap_err();
            assert!(err.to_string().contains("not allowed"), "{url}: {err}");
            // The scheme restriction applies even when private addresses are allowed.
            assert!(validate_parsed_url(&parse(url), true).is_err(), "{url}");
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
            assert!(
                validate_parsed_url(&parse(url), false).is_err(),
                "should be blocked: {url}"
            );
        }
    }

    #[test]
    fn test_allows_public_urls() {
        for url in [
            "https://example.com/api",
            "http://example.com:8080/x?y=z",
            "https://93.184.216.34/",
        ] {
            assert!(
                validate_parsed_url(&parse(url), false).is_ok(),
                "should be allowed: {url}"
            );
        }
    }

    #[test]
    fn test_allow_private_permits_internal_ips_but_not_schemes() {
        assert!(validate_parsed_url(&parse("http://127.0.0.1/"), true).is_ok());
        assert!(validate_parsed_url(&parse("http://192.168.1.1/"), true).is_ok());
        assert!(validate_parsed_url(&parse("file:///etc/passwd"), true).is_err());
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
                is_blocked_ip(ip.parse().unwrap()),
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
                !is_blocked_ip(ip.parse().unwrap()),
                "should be allowed: {ip}"
            );
        }
    }
}
