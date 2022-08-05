use crate::context::{Body, Receiver, Sender};
use crate::error::ErrorEnum;
use async_trait::async_trait;
use http::{Request, Response};
use std::future::Future;
use std::pin::Pin;
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
    ) -> Result<Response<Body>, ErrorEnum>;
}

pub struct Wrap {
    f: fn(&Request<Body>, &params::Params) -> FnOutput<Response<Body>>,
}

#[async_trait]
impl HttpRoute for Wrap {
    async fn handle(
        &self,
        req: &Request<Body>,
        params: &params::Params,
    ) -> Result<Response<Body>, ErrorEnum> {
        (self.f)(req, params).await
    }
}

pub fn wrap(f: fn(&Request<Body>, &params::Params) -> FnOutput<Response<Body>>) -> Wrap {
    Wrap { f }
}

pub type FnOutput<T> = Pin<Box<dyn Future<Output = Result<T, ErrorEnum>> + Send>>;
