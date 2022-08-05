use crate::codec::{http::Http, websocket::Ws};
use crate::context::{Body, Receiver, Sender};
use crate::error::ErrorEnum;
use base64::encode;
use http::header::{CONNECTION, CONTENT_TYPE, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, UPGRADE};
use http::{Request, Response, StatusCode};
use sha::sha1::Sha1;
use sha::utils::{Digest, DigestExt};
use std::fmt;

pub struct WsUpgrader;

#[derive(Debug)]
pub enum WsUpgradeError {
    UpgradeFailed,
}

impl std::fmt::Display for WsUpgradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpgradeFailed => f.write_str("websocket upgrade failed"),
        }
    }
}

impl std::error::Error for WsUpgradeError {}

impl WsUpgrader {
    const WS_KEY: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    pub async fn upgrade<'a>(
        mut tx: Sender<'a, Http<Body>>,
        rx: Receiver<'a, Http<Body>>,
        req: &Request<Body>,
    ) -> std::result::Result<(Sender<'a, Ws>, Receiver<'a, Ws>), ErrorEnum> {
        let headers = req.headers();

        if !headers.contains_key(CONNECTION) || !headers.contains_key(SEC_WEBSOCKET_KEY) {
            let resp: Response<Body> = Response::builder()
                .status(401)
                .header(CONTENT_TYPE, "text/plain")
                .body("400 Bad Request".into())
                .unwrap();

            tx.send(resp).await?;

            return Err(ErrorEnum::Other(Box::new(WsUpgradeError::UpgradeFailed)));
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
            .body(().into())
            .unwrap();

        tx.send(resp).await?;

        Ok((tx.to::<Ws>(), rx.to::<Ws>()))
    }
}
