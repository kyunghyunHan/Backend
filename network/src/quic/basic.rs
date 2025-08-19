use quinn::{Endpoint, ServerConfig, ClientConfig};
use std::sync::Arc;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

// 서버
async fn server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let endpoint = Endpoint::server(make_config(), "127.0.0.1:4433".parse()?)?;
    println!("🚀 QUIC 서버 시작됨!");
    
    let conn = endpoint.accept().await.unwrap().await?;
    println!("✅ 클라이언트 연결됨");
    
    let (mut send, mut recv) = conn.accept_bi().await?;
    
    let data = recv.read_to_end(1000).await?;
    println!("📥 받음: {}", String::from_utf8_lossy(&data));
    
    send.write_all(b"response from server").await?;
    send.finish()?;  // .await 제거
    println!("📤 응답 전송됨");
    
    Ok(())
}

// 클라이언트
async fn client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(make_client_config());
    
    let conn = endpoint.connect("127.0.0.1:4433".parse()?, "localhost")?.await?;
    println!("🔗 서버에 연결됨");
    
    let (mut send, mut recv) = conn.open_bi().await?;
    
    send.write_all(b"hello from client").await?;
    send.finish()?;  // .await 제거
    println!("📤 메시지 전송됨");
    
    let data = recv.read_to_end(1000).await?;
    println!("📥 응답 받음: {}", String::from_utf8_lossy(&data));
    
    Ok(())
}

#[tokio::main]
pub async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // CryptoProvider 설정 (에러 해결)
    rustls::crypto::ring::default_provider().install_default().unwrap();
    
    println!("=== QUIC 예제 시작 ===\n");
    
    let server_task = tokio::spawn(server());
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // 클라이언트 에러도 처리
    if let Err(e) = client().await {
        println!("⚠️ 클라이언트 에러: {}", e);
    }
    
    // 서버 에러도 처리
    if let Err(e) = server_task.await {
        println!("⚠️ 서버 태스크 에러: {}", e);
    }
    
    println!("\n=== QUIC 예제 완료 ===");
    Ok(())
}

// 서버 설정 - 완전히 간단하게
fn make_config() -> ServerConfig {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    
    // 올바른 방법으로 cert와 key 추출
    let cert_der = CertificateDer::from(cert.cert.der().to_vec());
    let key_der = PrivateKeyDer::Pkcs8(
        PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der())
    );
    
    ServerConfig::with_single_cert(vec![cert_der], key_der).unwrap()
}

// 클라이언트 설정 - 매우 간단하게
fn make_client_config() -> ClientConfig {
    #[derive(Debug)]
    struct SkipVerify;
    
    impl rustls::client::danger::ServerCertVerifier for SkipVerify {
        fn verify_server_cert(
            &self,
            _end_entity: &CertificateDer<'_>,
            _intermediates: &[CertificateDer<'_>],
            _server_name: &rustls::pki_types::ServerName<'_>,
            _ocsp_response: &[u8],
            _now: rustls::pki_types::UnixTime,
        ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        }
        
        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }
        
        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }
        
        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            vec![
                rustls::SignatureScheme::RSA_PKCS1_SHA256,
                rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
                rustls::SignatureScheme::RSA_PSS_SHA256,
                rustls::SignatureScheme::ED25519,
                rustls::SignatureScheme::RSA_PKCS1_SHA384,
                rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
                rustls::SignatureScheme::RSA_PSS_SHA384,
                rustls::SignatureScheme::RSA_PKCS1_SHA512,
                rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
                rustls::SignatureScheme::RSA_PSS_SHA512,
            ]
        }
    }
    
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipVerify))
        .with_no_client_auth();
    
    // 올바른 방법으로 ClientConfig 생성
    ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto).unwrap()
    ))
}

// Cargo.toml:
/*
[dependencies]
quinn = "0.11"
rustls = "0.23"
tokio = { version = "1.0", features = ["full"] }
rcgen = "0.12"
*/