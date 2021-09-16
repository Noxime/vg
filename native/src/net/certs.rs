use std::sync::Arc;

use quinn::ClientConfig;
use rustls::ServerCertVerified;
use tracing::warn;

// This verifier just accepts any certificate as ok. Not safe, but good for dev
struct SkipCertificationVerification;

impl rustls::ServerCertVerifier for SkipCertificationVerification {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<ServerCertVerified, rustls::TLSError> {
        Ok(ServerCertVerified::assertion())
    }
}

pub fn insecure_client() -> ClientConfig {
    warn!("Using unverified certificate configuration. Server connection is insecure!");
    let mut cfg = quinn::ClientConfigBuilder::default().build();

    // Get a mutable reference to the 'crypto' config in the 'client config'.
    let tls_cfg: &mut rustls::ClientConfig =
        std::sync::Arc::get_mut(&mut cfg.crypto).unwrap();

    // Change the certification verifier.
    // This is only available when compiled with the 'dangerous_configuration' feature.
    tls_cfg
        .dangerous()
        .set_certificate_verifier(Arc::new(SkipCertificationVerification));
    cfg
}

pub fn generate_self_signed_cert(hostnames: Vec<String>) -> Result<(quinn::Certificate, quinn::PrivateKey), super::Error> {
    warn!("Using self-signed certificates for {:?}, vulnerable to MitM", hostnames);
    // Generate dummy certificate.
    let certificate = rcgen::generate_simple_self_signed(hostnames)?;
    let serialized_key = certificate.serialize_private_key_der();
    let serialized_certificate = certificate.serialize_der()?;

    // Write to files.
    // fs::write(&cert_path, &serialized_certificate).context("failed to write certificate")?;
    // fs::write(&key_path, &serialized_key).context("failed to write private key")?;

    let cert = quinn::Certificate::from_der(&serialized_certificate)?;
    let key = quinn::PrivateKey::from_der(&serialized_key)?;
    Ok((cert, key))
}
