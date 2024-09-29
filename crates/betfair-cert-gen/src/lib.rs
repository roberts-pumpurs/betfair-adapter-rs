//! This crate provides functionality to generate Betfair certificates for non-interactive bot usage.
//! It includes methods for creating SSL/TLS certificates and key pairs.

use rand::rngs::OsRng;
use rcgen::{
    date_time_ymd, Certificate, CertificateParams, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose,
};
use rsa::pkcs8::EncodePrivateKey;
use rsa::RsaPrivateKey;
use eyre::WrapErr;

/// Generate a Betfair certificate for non-interactive bot usage
/// Reference:
/// <https://docs.developer.betfair.com/display/1smk3cen4v3lu3yomq5qye0ni/Non-Interactive+(bot)+login>
///
/// Generates an SSL/TLS certificate and key pair using the provided information.
///
/// # Arguments
///
/// * `country_name` - The two-letter country code (e.g., "US")
/// * `state_or_province_name` - The state or province name
/// * `organizational_unit_name` - The name of the organizational unit
/// * `common_name` - The common name (typically the domain name)
///
/// # Returns
///
/// Returns a Result containing a tuple of `(Certificate, KeyPair)` on success.
///
/// # Errors
///
/// This function may return an error in the following situations:
///
/// - If any of the input strings contain invalid characters for a certificate.
/// - If the certificate generation process fails internally.
/// - If there's insufficient entropy for secure key generation.
/// - If the system runs out of memory during the generation process.
///
/// The returned error will be of type `eyre::Report`, which may contain additional
/// context about the nature of the error.
///
/// # Panics
///
/// This function may panic in the following situations:
///
/// - If the underlying cryptographic libraries fail in an unexpected way.
/// - If there's a critical failure in memory allocation during the generation process.
/// - If the function is called in an environment where required system calls are not available.
///
/// Note: While efforts are made to return errors instead of panicking, some underlying
/// libraries or extreme conditions may still cause panics.
///
/// # Example
///
/// ```
/// use betfair_cert_gen::rcgen_cert;
///
/// let result = rcgen_cert("US", "California", "IT Department", "example.com");
/// match result {
///     Ok((cert, key_pair)) => println!("Certificate generated successfully"),
///     Err(e) => eprintln!("Failed to generate certificate: {}", e),
/// }
/// ```
pub fn rcgen_cert(
    country_name: &str,
    state_or_province_name: &str,
    organizational_unit_name: &str,
    common_name: &str,
) -> eyre::Result<(Certificate, KeyPair)> {
    let mut params: CertificateParams = CertificateParams::default();
    params.not_before = date_time_ymd(2021, 5, 19);
    params.not_after = date_time_ymd(4096, 1, 1);
    params.distinguished_name = DistinguishedName::new();
    params.is_ca = IsCa::ExplicitNoCa;
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::CountryName, country_name);
    params
        .distinguished_name
        .push(DnType::StateOrProvinceName, state_or_province_name);
    params
        .distinguished_name
        .push(DnType::OrganizationalUnitName, organizational_unit_name);
    params
        .distinguished_name
        .push(DnType::CommonName, common_name);

    let private_key = RsaPrivateKey::new(&mut OsRng, 2048)?;
    let private_key_der = private_key.to_pkcs8_der()?;
    let key_pair = rcgen::KeyPair::try_from(private_key_der.as_bytes()).wrap_err("failed to create keypair")?;
    let cert = params.self_signed(&key_pair)?;
    Ok((cert, key_pair))
}
