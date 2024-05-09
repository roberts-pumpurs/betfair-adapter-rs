use std::sync::Arc;

use rcgen::generate_simple_self_signed;
use rustls::pki_types::PrivateKeyDer;
use rustls::ServerConfig;

pub fn generate_cert() -> Result<(String, String), rcgen::RcgenError> {
    let subject_alt_names = vec!["localhost".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names)?;
    let cert_pem = cert.cert.pem();
    let key_pem = cert.key_pair.serialize_pem();
    Ok((cert_pem, key_pem))
}

pub fn rustls_config(cert_pem: &str, key_pem: &str) -> Arc<ServerConfig> {
    let cert = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .next()
        .unwrap()
        .unwrap();
    let certs = vec![cert];
    let key = rustls_pemfile::pkcs8_private_keys(&mut key_pem.as_bytes())
        .next()
        .unwrap()
        .unwrap();

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, PrivateKeyDer::Pkcs8(key))
        .unwrap();

    Arc::new(config)
}
