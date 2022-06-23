use crate::codec::prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::context::Body;
use base64::encode;
use bytes::BytesMut;
use http::header::{CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, UPGRADE};
use http::{Request, Response, StatusCode};
use sha::sha1::Sha1;
use sha::utils::{Digest, DigestExt};

pub struct WsUpgrader;

impl WsUpgrader {
    const WS_KEY: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    pub async fn upgrade(stream: &mut TcpStream, req: &Request<Body>) -> std::io::Result<()> {
        let headers = req.headers();
        let mut buf = BytesMut::new();

        if !headers.contains_key(CONNECTION) || !headers.contains_key(SEC_WEBSOCKET_KEY) {
            let resp: Response<Body> = Response::builder()
                .status(401)
                .header("Content-Type", "text/plain")
                .body("400 Bad Request".into())
                .unwrap();

            let mut http: Http<_> = Http::new();
            http.encode(resp, &mut buf).unwrap();
            stream.write(&buf).await?;

            return Err(std::io::Error::other("upgrade failed"));
        }

        let mut builder = Response::builder()
            .header(UPGRADE, "websocket")
            .header(CONNECTION, "Upgrade");

        let key = format!(
            "{}{}",
            &headers[SEC_WEBSOCKET_KEY].to_str().unwrap(),
            Self::WS_KEY
        );

        let len_a = &headers[SEC_WEBSOCKET_KEY].len();
        let len_b = Self::WS_KEY.len();

        let mut bytes = Vec::with_capacity(len_a + len_b);
        bytes.extend_from_slice(headers[SEC_WEBSOCKET_KEY].as_bytes());
        bytes.extend_from_slice(Self::WS_KEY.as_bytes());

        let hashed = Sha1::default().digest(key.as_bytes()).to_bytes();
        let encoded = encode(hashed);
        builder = builder.header(SEC_WEBSOCKET_ACCEPT, encoded);

        let resp = builder
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .body(())
            .unwrap();

        let mut http: Http<()> = Http::new();
        http.encode(resp, &mut buf).unwrap();
        stream.write(&buf).await?;

        Ok(())
    }
}
