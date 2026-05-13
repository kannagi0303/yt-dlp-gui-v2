use std::io::Read;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

const THUMBNAIL_TIMEOUT: Duration = Duration::from_secs(12);
const MAX_THUMBNAIL_BYTES: u64 = 12 * 1024 * 1024;

pub struct ThumbnailFetchEvent {
    pub key: String,
    pub result: Result<eframe::egui::ColorImage, String>,
}

pub fn run_thumbnail_fetch_worker(
    key: String,
    url: String,
    proxy_url: String,
    no_check_certificates: bool,
    result_tx: Sender<ThumbnailFetchEvent>,
) {
    thread::spawn(move || {
        let result = fetch_thumbnail_image(&url, &proxy_url, no_check_certificates);
        let _ = result_tx.send(ThumbnailFetchEvent { key, result });
    });
}

fn fetch_thumbnail_image(
    url: &str,
    proxy_url: &str,
    no_check_certificates: bool,
) -> Result<eframe::egui::ColorImage, String> {
    let bytes = fetch_thumbnail_bytes(url, proxy_url, no_check_certificates)?;
    let image = image::load_from_memory(&bytes)
        .map_err(|error| format!("Thumbnail decode failed: {error}"))?
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.as_flat_samples();
    Ok(eframe::egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

fn fetch_thumbnail_bytes(
    url: &str,
    proxy_url: &str,
    no_check_certificates: bool,
) -> Result<Vec<u8>, String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("Thumbnail load failed: empty URL".to_owned());
    }

    let mut builder = ureq::builder()
        .timeout(THUMBNAIL_TIMEOUT)
        .user_agent("yt-dlp-gui-v2 thumbnail loader");

    let proxy_url = proxy_url.trim();
    if !proxy_url.is_empty() {
        let proxy = ureq::Proxy::new(proxy_url)
            .map_err(|error| format!("Invalid thumbnail proxy setting: {error}"))?;
        builder = builder.proxy(proxy);
    }

    if no_check_certificates {
        builder = builder.tls_config(Arc::new(insecure_tls_config()));
    }

    let agent = builder.build();
    let response = agent.get(url).call().map_err(format_ureq_error)?;

    let mut reader = response.into_reader().take(MAX_THUMBNAIL_BYTES + 1);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Thumbnail load failed: {error}"))?;

    if bytes.is_empty() {
        return Err("Thumbnail load failed: no data received".to_owned());
    }
    if bytes.len() as u64 > MAX_THUMBNAIL_BYTES {
        return Err("Thumbnail load failed: image too large".to_owned());
    }

    Ok(bytes)
}

fn format_ureq_error(error: ureq::Error) -> String {
    match error {
        ureq::Error::Status(code, _) => format!("Thumbnail load failed: HTTP {code}"),
        ureq::Error::Transport(error) => format!("Thumbnail load failed: {error}"),
    }
}

fn insecure_tls_config() -> rustls::ClientConfig {
    rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAnyServerCert))
        .with_no_client_auth()
}

#[derive(Debug)]
struct AcceptAnyServerCert;

impl rustls::client::danger::ServerCertVerifier for AcceptAnyServerCert {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
        ]
    }
}
