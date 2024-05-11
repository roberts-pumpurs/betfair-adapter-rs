use rand::rngs::OsRng;
use rcgen::{
    date_time_ymd, Certificate, CertificateParams, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose,
};
use rsa::pkcs8::EncodePrivateKey;
use rsa::RsaPrivateKey;

/// Generate a Betfair certificate for non-interactive bot usage
/// Reference:
/// https://docs.developer.betfair.com/display/1smk3cen4v3lu3yomq5qye0ni/Non-Interactive+(bot)+login
pub fn rcgen_cert(
    country_name: &str,
    state_or_province_name: &str,
    organizational_unit_name: &str,
    common_name: &str,
) -> eyre::Result<(Certificate, KeyPair)> {
    let mut params: CertificateParams = Default::default();
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
    let key_pair = rcgen::KeyPair::try_from(private_key_der.as_bytes()).unwrap();
    let cert = params.self_signed(&key_pair)?;

    Ok((cert, key_pair))
}
