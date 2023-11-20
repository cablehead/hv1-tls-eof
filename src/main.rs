use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let accept_tls = configure_tls(PEM);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let stream = match accept_tls.accept(stream).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error accepting TLS connection: {:?}", e);
                continue;
            }
        };

        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = hyper::server::conn::http2::Builder::new(
                hyper_util::rt::tokio::TokioExecutor::new(),
            )
            .serve_connection(io, service_fn(hello))
            .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

use std::io::Cursor;

fn configure_tls(pem: &str) -> tokio_rustls::TlsAcceptor {
    let mut pem = Cursor::new(pem);
    let items = rustls_pemfile::read_all(&mut pem).unwrap();

    let certs: Vec<rustls::Certificate> = items
        .iter()
        .filter_map(|item| match item {
            rustls_pemfile::Item::X509Certificate(cert) => Some(rustls::Certificate(cert.to_vec())),
            _ => None,
        })
        .collect();

    let key = items
        .into_iter()
        .find_map(|item| match item {
            rustls_pemfile::Item::RSAKey(key) => Some(rustls::PrivateKey(key)),
            rustls_pemfile::Item::PKCS8Key(key) => Some(rustls::PrivateKey(key)),
            rustls_pemfile::Item::ECKey(key) => Some(rustls::PrivateKey(key)),
            rustls_pemfile::Item::X509Certificate(_) => None,
            _ => todo!(),
        })
        .unwrap();

    let mut config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(config))
}

/* generated with:

openssl req -x509 -newkey rsa:4096 \
    -days 365 -nodes -subj "/CN=localhost" \
    -keyout /dev/stdout -out /dev/stdout | bp

*/
const PEM: &str = r#"-----BEGIN PRIVATE KEY-----
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
qTX8U2QktajNPyq4LhCtSQwCiKa5Ids=
-----END PRIVATE KEY-----
-----BEGIN CERTIFICATE-----
MIIFCTCCAvGgAwIBAgIUO+D3FMRJO4XLrLtsj4jbE3jvPdswDQYJKoZIhvcNAQEL
BQAwFDESMBAGA1UEAwwJbG9jYWxob3N0MB4XDTIzMTExNzE5MDkyN1oXDTI0MTEx
NjE5MDkyN1owFDESMBAGA1UEAwwJbG9jYWxob3N0MIICIjANBgkqhkiG9w0BAQEF
AAOCAg8AMIICCgKCAgEAoeI+FLaH0EBE9iHzDju1VjUylVvaf+EAQHB4SGwmm2WQ
/LsdS0MGfza+QKZ0WBmOS3pRjC1umReVJju3b7M0EiYgGN4+uog0Ge1dbIO+DTWR
EBAk1vp7uBEOGGRnqs3IFE/xLIl1YFG7EjeoDvIBjXRLNCeFS8KlC+UHUUc314Ti
4nLZb9mLINPZiKBqpGT1yDtmR1C/vF2FwGngYc2Et7hlcOQboFBqWEx3RyYSpkSj
ZiaP0encL1xFoS/a4jHZkZfaqNBjxewuXMXqkm7UwwSSKfv+lsiuxcyVfqSAt7/m
Zz69f309Mf/vd1JvX5Cer8CZW+SPrzNBVATx4eZJvBYSJkPTiv9dLhD/dBn1CndI
Gc3yUUmrSbvGmLR8zLx3m1tI5oDqtqpdWV8vTGCbunrt/V6xmWC9zRxTC5i86rsV
UdXWQ9IioUMZxCBqagJGKTQINZNUqOpLBcFyVkmYgsIEJv3MPgYq/Y8qR/ZP+dSO
jllV71jVGYOE9dISV3U8b+pmZ7eRlpcSYmaV1pbKbZ0uPm5uBypVARPoLaHmUDxp
xhmBXucMs91qO0o36/E0G7iccvzBow5SFl3KTCXdPBLn+TX2uB74jPi+hHpAgVtA
A444K6RTrqBMMMlP0HChWlBMG/Ai6cF47Z3bXzyyJDwqtTOY/1JwFmL2vHKYMtcC
AwEAAaNTMFEwHQYDVR0OBBYEFL3fAX6OwJjk5VROhQrHv2dGOvSZMB8GA1UdIwQY
MBaAFL3fAX6OwJjk5VROhQrHv2dGOvSZMA8GA1UdEwEB/wQFMAMBAf8wDQYJKoZI
hvcNAQELBQADggIBAGHql7PQpr1gG17mrqE9r1PEdp+OdGiiduzsMZaRLtoCFEZ8
FQD2SxXr/AETWVrUpoe6097t35OhmDXy03+1NauiyFAUIR3vKILpKrz0KFP7yhZq
qqF+LwjlbrQktSG0AcZlynXAMEn+QKmKV31lzkWUl0JycRCmCyYDJgC5LXiGlugF
EbxqbFn66TgVjM1TbIRFCOptLlF3Itnz3ZE5hC7bz6/xqxU8MZ76bNSqH2ZSNfRR
tTUNC24P7E4CZqz43EXv/OSfgFqDJJza2YkdVW4O7iyPNyDSjz+1rK6Ho7Cy4XnO
+L6s2fCTAQUyOB/BeI0t3/tl4h3IPxxF/rh63hGrMStMBWIM2kb7aNxsTKu+pDqD
UAr2oL+74RjQFStQTnJFPJPyRuBpp7EOJzaY266fFYEZ7EbobL5biU5Tm3/vaLKf
O1K+XUk+ljxZKk5gTccFvaKrs9KHahN+RXgF7jVnwNvP+54+V1DsfzkLgoYu0fcS
clH26IAhCL8mh5neR3Y26hhDgu+1bGFYACurpa1LqE5fx5PE513zyhEbE3ObmiJQ
iDPy67ZBPkm83CAe77uggJZEYbZgaHmL/fB4tOGTB0ih9y7yqpUQ31XticDEjVTZ
utSlHzqjh4tkqZWFPCDmnfhQUcPWJ7wp1zNPgO2n1Qi0TfsQJK6x055ffto7
-----END CERTIFICATE-----"#;
