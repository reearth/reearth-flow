//! HTTP client with network egress control.
//!
//! The engine executes user-authored workflows server-side, so an unrestricted
//! HTTP client is a server-side request forgery primitive: cloud metadata
//! endpoints, loopback services and private-network hosts would all be
//! reachable from URLs taken from workflow data. Outbound HTTP requests made on
//! behalf of a workflow should go through the clients in this crate, which
//! enforce the restriction in three layers:
//!
//! 1. Before sending, the URL scheme must be http/https and a literal-IP host
//!    must not be a blocked address.
//! 2. Every hostname resolution (initial request, every redirect hop, every
//!    retry) drops blocked addresses, which also closes DNS-rebinding gaps
//!    because filtering happens at connection time.
//! 3. Literal-IP redirect targets never hit DNS, so each hop is checked by the
//!    redirect policy as well.
//!
//! The layers are built into [`HttpClient`] and [`BlockingHttpClient`] and the
//! underlying `reqwest` clients are never exposed, so there is no unguarded
//! route. Proxies are disabled because a proxy would make the resolver see
//! only the proxy host.
//!
//! Self-hosted deployments that legitimately call services on private
//! addresses can opt out with `FLOW_RUNTIME_HTTP_ALLOW_PRIVATE_NETWORK=true`.
//! The scheme restriction and the block on never-routable addresses
//! (unspecified, broadcast, multicast) always apply.

mod body;
mod client;
mod error;
mod policy;
#[cfg(feature = "xml-schema")]
mod schema;

pub use body::read_to_end_capped;
pub use client::{
    BlockingHttpClient, BlockingRequestBuilder, HttpClient, HttpClientBuilder, RequestBuilder,
};
pub use error::EgressError;
pub use policy::{EgressPolicy, ALLOW_PRIVATE_NETWORK_ENV};
#[cfg(feature = "xml-schema")]
pub use schema::{SchemaDiskCache, SCHEMA_CACHE_DIR_ENV};
