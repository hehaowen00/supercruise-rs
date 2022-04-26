use crate::codec::http::*;
use crate::codec::websocket::*;
use crate::context::Body;
use crate::context::Context;
use crate::ws::WsUpgrader;

use async_trait::async_trait;
use http::{Request, Response};
use std::future::Future;
use tokio::net::TcpStream;

#[async_trait]
pub trait ServiceFactory: Send + Sync {
    async fn next(&mut self) -> Result<Box<dyn Endpoint>, ()>;
}

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
    async fn handle(&self, ctx: &mut Context<Codec>) -> std::io::Result<()>;
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

#[async_trait]
impl<F, R> Route<Http<Body>> for F
where
    F: Fn(&mut Context<Http<Body>>) -> R + Send + Sync,
    R: Future<Output = std::io::Result<()>> + Send,
{
    async fn handle(&self, ctx: &mut Context<Http<Body>>) -> std::io::Result<()> {
        self(ctx).await
    }
}

pub struct HttpEndpoint {
    handler: Box<dyn HttpRoute>,
}

impl HttpEndpoint {
    pub fn new<H>(handler: H) -> Self
    where
        H: HttpRoute + Send + Sync + 'static,
    {
        Self {
            handler: Box::new(handler),
        }
    }
}

#[async_trait]
impl Endpoint for HttpEndpoint {
    async fn handle(
        &self,
        mut stream: TcpStream,
        req: Request<Body>,
    ) -> std::io::Result<Option<TcpStream>> {
        let mut context: Context<Http<_>> = Context::from(&mut stream);

        let resp = self.handler.handle(req).await?;
        context.send(resp).await?;

        Ok(None)
    }
}

pub struct WsEndpoint {
    handler: Box<dyn Route<Ws> + Send + Sync>,
}

impl WsEndpoint {
    pub fn new(handler: impl Route<Ws> + Send + Sync + 'static) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }
}

#[async_trait]
impl Endpoint for WsEndpoint {
    async fn handle(
        &self,
        mut stream: TcpStream,
        req: Request<Body>,
    ) -> std::io::Result<Option<TcpStream>> {
        WsUpgrader::upgrade(&mut stream, req).await?;

        let mut context = Context::<Ws>::from(&mut stream);

        self.handler.handle(&mut context).await?;

        let close = WsFrame::builder().close();

        context.send(close).await?;

        Ok(None)
    }
}
