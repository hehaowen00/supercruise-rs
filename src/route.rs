use crate::codec::websocket::*;
use crate::context::{Body, Receiver, Sender};

use async_trait::async_trait;
use http::{Request, Response};
use std::future::Future;
use tokio::net::TcpStream;

#[async_trait]
pub trait Endpoint: Send + Sync {
    async fn handle(
        &self,
        stream: TcpStream,
        req: Request<Body>,
    ) -> std::io::Result<Option<TcpStream>>;
}

#[async_trait]
pub trait Route<Codec> {
    async fn handle(&self, tx: &mut Sender<Ws>, rx: &mut Receiver<Ws>) -> std::io::Result<()>;
}

#[async_trait]
pub trait HttpRoute: Send + Sync {
    async fn handle(&self, req: Request<Body>) -> std::io::Result<Response<Body>>;
}

#[async_trait]
impl<F, R> HttpRoute for F
where
    F: Fn(Request<Body>) -> R + Send + Sync,
    R: Future<Output = std::io::Result<Response<Body>>> + Send,
{
    async fn handle(&self, req: Request<Body>) -> std::io::Result<Response<Body>> {
        self(req).await
    }
}
