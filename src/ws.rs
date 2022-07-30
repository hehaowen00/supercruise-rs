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
        if !headers.contains_key(CONNECTION) || !headers.contains_key(SEC_WEBSOCKET_KEY) {
            return Err(std::io::Error::other("upgrade failed"));
        }

        let mut http: Http<()> = Http::new();
        let mut builder = Response::builder();
        builder = builder.header(UPGRADE, "websocket");
        builder = builder.header(CONNECTION, "Upgrade");

        let key = format!(
            "{}{}",
            &headers[SEC_WEBSOCKET_KEY].to_str().unwrap(),
            Self::WS_KEY
        );
        let hashed = Sha1::default().digest(key.as_bytes()).to_bytes();
        let encoded = encode(hashed);
        builder = builder.header(SEC_WEBSOCKET_ACCEPT, encoded);

        let resp = builder
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .body(())
            .unwrap();

        let mut buf = BytesMut::new();

        http.encode(resp, &mut buf).unwrap();
        stream.write(&buf).await?;

        Ok(())
    }
}
