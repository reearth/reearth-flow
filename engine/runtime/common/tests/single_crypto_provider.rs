// Its own test binary so no other test has installed a provider first.
//
// gRPC clients (tonic, under the Pub/Sub client) ask rustls for a default
// provider without installing one. That works only while exactly one provider
// is compiled in; with two, rustls panics and the worker dies at startup.
// `cargo make test-rs` builds the whole workspace at once, so this fails if any
// crate turns on a second provider (e.g. reqwest's `rustls` feature pulls in
// aws-lc-rs; use `rustls-no-provider`).
#[test]
fn rustls_resolves_a_provider_from_crate_features() {
    let _ = rustls::ClientConfig::builder();
}
