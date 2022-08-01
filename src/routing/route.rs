use crate::context::{Body, Receiver, Sender};
use async_trait::async_trait;
use http::{Request, Response};
use trie_rs::path::params;

#[async_trait]
pub trait Route<Codec> {
    async fn handle(
        &self,
        tx: &mut Sender<Codec>,
        rx: &mut Receiver<Codec>,
        params: &params::Params,
    ) -> std::io::Result<()>;
}

#[async_trait]
pub trait HttpRoute: Send + Sync {
    async fn handle(
        &self,
        req: &Request<Body>,
        params: &params::Params,
    ) -> std::io::Result<Response<Body>>;
}
