/// Install ring as the process-wide rustls crypto provider.
///
/// rustls cannot pick a provider on its own once more than one is compiled
/// in, and the dependency tree does not guarantee there is only one, so every
/// TLS client is built on the provider installed here. reqwest is built with
/// no provider of its own, so call this before building a reqwest client, and
/// at the start of any binary that opens TLS connections through other crates
/// (e.g. the gRPC Pub/Sub client). Safe to call more than once.
pub fn install_crypto_provider() {
    // Fails only if a provider is already installed, which is what we want.
    let _ = rustls::crypto::ring::default_provider().install_default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_a_default_provider() {
        install_crypto_provider();
        install_crypto_provider();
        assert!(rustls::crypto::CryptoProvider::get_default().is_some());
    }
}
