use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use reqwest::{redirect, Method};

use crate::error::EgressError;
use crate::policy::EgressPolicy;

/// Builds clients that always carry the egress guard. Nothing here can remove
/// the guarded resolver, the redirect policy or the proxy bypass.
#[derive(Debug, Clone)]
pub struct HttpClientBuilder {
    policy: EgressPolicy,
    connect_timeout: Option<Duration>,
    timeout: Option<Duration>,
    user_agent: Option<String>,
    verify_ssl: bool,
    max_redirects: Option<usize>,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Applies the guard and the shared settings to either reqwest builder type.
macro_rules! configure {
    ($builder:expr, $config:expr) => {{
        let config = $config;
        let mut builder = $builder
            .dns_resolver(Arc::new(GuardedResolver {
                policy: config.policy,
            }))
            .redirect(redirect_policy(config.max_redirects, config.policy))
            .no_proxy()
            .danger_accept_invalid_certs(!config.verify_ssl);
        if let Some(d) = config.connect_timeout {
            builder = builder.connect_timeout(d);
        }
        if let Some(d) = config.timeout {
            builder = builder.timeout(d);
        }
        if let Some(ua) = config.user_agent {
            builder = builder.user_agent(ua);
        }
        builder
    }};
}

impl HttpClientBuilder {
    pub fn new() -> Self {
        Self {
            policy: EgressPolicy::from_env(),
            connect_timeout: None,
            timeout: None,
            user_agent: None,
            verify_ssl: true,
            max_redirects: Some(10),
        }
    }

    pub fn policy(mut self, policy: EgressPolicy) -> Self {
        self.policy = policy;
        self
    }

    pub fn connect_timeout(mut self, d: Duration) -> Self {
        self.connect_timeout = Some(d);
        self
    }

    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = Some(d);
        self
    }

    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn verify_ssl(mut self, verify: bool) -> Self {
        self.verify_ssl = verify;
        self
    }

    /// `None` disables redirects.
    pub fn max_redirects(mut self, n: Option<usize>) -> Self {
        self.max_redirects = n;
        self
    }

    /// Must not be called (or the client dropped) on a tokio worker thread:
    /// the blocking client owns its own runtime.
    pub fn build_blocking(self) -> Result<BlockingHttpClient, EgressError> {
        let policy = self.policy;
        let inner = configure!(reqwest::blocking::Client::builder(), self)
            .build()
            .map_err(EgressError::Build)?;
        Ok(BlockingHttpClient { inner, policy })
    }

    pub fn build(self) -> Result<HttpClient, EgressError> {
        let policy = self.policy;
        let inner = configure!(reqwest::Client::builder(), self)
            .build()
            .map_err(EgressError::Build)?;
        Ok(HttpClient { inner, policy })
    }
}

/// Blocking client with the egress guard built in.
#[derive(Debug, Clone)]
pub struct BlockingHttpClient {
    inner: reqwest::blocking::Client,
    policy: EgressPolicy,
}

impl BlockingHttpClient {
    pub fn request(
        &self,
        method: Method,
        url: &str,
    ) -> Result<reqwest::blocking::RequestBuilder, EgressError> {
        let url = self.policy.parse_url(url)?;
        Ok(self.inner.request(method, url))
    }

    pub fn get(&self, url: &str) -> Result<reqwest::blocking::RequestBuilder, EgressError> {
        self.request(Method::GET, url)
    }

    /// Re-checks the URL, since a built request's URL can be rewritten.
    pub fn execute(
        &self,
        request: reqwest::blocking::Request,
    ) -> Result<reqwest::blocking::Response, EgressError> {
        self.policy.check_url(request.url())?;
        self.inner.execute(request).map_err(EgressError::Request)
    }
}

/// Async client with the egress guard built in.
#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
    policy: EgressPolicy,
}

impl HttpClient {
    pub fn request(
        &self,
        method: Method,
        url: &str,
    ) -> Result<reqwest::RequestBuilder, EgressError> {
        let url = self.policy.parse_url(url)?;
        Ok(self.inner.request(method, url))
    }

    pub fn get(&self, url: &str) -> Result<reqwest::RequestBuilder, EgressError> {
        self.request(Method::GET, url)
    }

    /// Re-checks the URL, since a built request's URL can be rewritten.
    pub async fn execute(
        &self,
        request: reqwest::Request,
    ) -> Result<reqwest::Response, EgressError> {
        self.policy.check_url(request.url())?;
        self.inner
            .execute(request)
            .await
            .map_err(EgressError::Request)
    }
}

/// Drops blocked addresses from every resolution, so a hostname pointing (or
/// re-pointing) at an internal address can never be connected to.
#[derive(Debug, Clone)]
struct GuardedResolver {
    policy: EgressPolicy,
}

impl Resolve for GuardedResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let policy = self.policy;
        let host = name.as_str().to_string();
        Box::pin(async move {
            let addrs: Vec<SocketAddr> = tokio::task::spawn_blocking({
                let host = host.clone();
                move || (host.as_str(), 0).to_socket_addrs().map(Vec::from_iter)
            })
            .await
            .map_err(boxed)?
            .map_err(boxed)?;
            if addrs.is_empty() {
                return Err(boxed(std::io::Error::other(format!(
                    "no addresses found for '{host}'"
                ))));
            }

            let ips: Vec<IpAddr> = addrs.iter().map(|addr| addr.ip()).collect();
            let allowed: Vec<SocketAddr> = addrs
                .into_iter()
                .filter(|addr| !policy.is_blocked(addr.ip()))
                .collect();
            if allowed.is_empty() {
                return Err(boxed(policy.blocked_error(&host, &ips)));
            }
            Ok(Box::new(allowed.into_iter()) as Addrs)
        })
    }
}

fn boxed(
    e: impl std::error::Error + Send + Sync + 'static,
) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(e)
}

/// Enforces the hop limit plus per-hop scheme and literal-IP checks. Hostname
/// hops are covered by the guarded resolver.
fn redirect_policy(max_redirects: Option<usize>, policy: EgressPolicy) -> redirect::Policy {
    let Some(max_redirects) = max_redirects else {
        return redirect::Policy::none();
    };
    redirect::Policy::custom(move |attempt| {
        if attempt.previous().len() > max_redirects {
            return attempt.error(format!("too many redirects (limit: {max_redirects})"));
        }
        if let Err(e) = policy.check_url(attempt.url()) {
            return attempt.error(e);
        }
        attempt.follow()
    })
}

#[cfg(test)]
mod tests {
    use std::io::{ErrorKind, Read, Write};
    use std::net::TcpListener;

    use super::*;
    use crate::error::{is_blocked, with_sources};

    const STRICT: EgressPolicy = EgressPolicy {
        allow_private_network: false,
    };

    fn idle_listener(addr: &str) -> TcpListener {
        let listener = TcpListener::bind(addr).unwrap();
        listener.set_nonblocking(true).unwrap();
        listener
    }

    fn assert_no_connection(listener: &TcpListener) {
        match listener.accept() {
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            other => panic!("expected no connection, got {other:?}"),
        }
    }

    fn strict_client() -> BlockingHttpClient {
        HttpClientBuilder::new()
            .policy(STRICT)
            .build_blocking()
            .unwrap()
    }

    #[test]
    fn test_get_to_loopback_is_blocked_before_connecting() {
        let listener = idle_listener("127.0.0.1:0");
        let url = format!("http://{}/", listener.local_addr().unwrap());

        let err = strict_client().get(&url).unwrap_err();

        assert!(matches!(err, EgressError::Blocked(_)), "{err}");
        assert_no_connection(&listener);
    }

    #[test]
    fn test_execute_rechecks_rewritten_url() {
        let listener = idle_listener("127.0.0.1:0");
        let client = strict_client();
        let mut request = client.get("http://example.com/").unwrap().build().unwrap();
        *request.url_mut() = format!("http://{}/", listener.local_addr().unwrap())
            .parse()
            .unwrap();

        let err = client.execute(request).unwrap_err();

        assert!(matches!(err, EgressError::Blocked(_)), "{err}");
        assert_no_connection(&listener);
    }

    #[tokio::test]
    async fn test_resolver_drops_loopback() {
        let resolver = GuardedResolver { policy: STRICT };

        let result = resolver.resolve("localhost".parse().unwrap()).await;

        let err = result.err().expect("localhost must not resolve");
        assert!(err.to_string().contains("is blocked"), "{err}");
    }

    #[test]
    fn test_redirect_to_never_routable_address_is_blocked() {
        // Private networking is allowed so the test server on loopback is
        // reachable; the redirect target stays blocked regardless.
        let target = idle_listener("0.0.0.0:0");
        let target_port = target.local_addr().unwrap().port();
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", server.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = server.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let response = format!(
                "HTTP/1.1 302 Found\r\nLocation: http://0.0.0.0:{target_port}/\r\n\
                 Content-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        let client = HttpClientBuilder::new()
            .policy(EgressPolicy {
                allow_private_network: true,
            })
            .build_blocking()
            .unwrap();

        let err = client.get(&url).unwrap().send().unwrap_err();

        handle.join().unwrap();
        assert!(with_sources(&err).contains("is blocked"), "{err:?}");
        assert!(is_blocked(&err), "{err:?}");
        assert_no_connection(&target);
    }

    #[test]
    fn test_send_blocked_by_resolver_is_detected() {
        let err = strict_client()
            .get("http://localhost:1/")
            .unwrap()
            .send()
            .unwrap_err();

        assert!(is_blocked(&err), "{err:?}");
    }

    #[test]
    fn test_connection_failure_is_not_blocked() {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let client = HttpClientBuilder::new()
            .policy(EgressPolicy {
                allow_private_network: true,
            })
            .build_blocking()
            .unwrap();

        let err = client
            .get(&format!("http://127.0.0.1:{port}/"))
            .unwrap()
            .send()
            .unwrap_err();

        assert!(!is_blocked(&err), "{err:?}");
    }
}
