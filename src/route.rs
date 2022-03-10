use crate::codec::http::*;
use crate::codec::websocket::*;
use crate::flow::Context;
use crate::ws::WsUpgrader;
use crate::context::Body;

use async_trait::async_trait;
use http::Request;
use tokio::net::TcpStream;

#[async_trait]
pub trait ServiceFactory: Send + Sync {
    async fn next(&mut self) -> Result<Box<dyn Endpoint>, ()>;
}

#[async_trait]
pub trait Endpoint: Send + Sync {
    async fn handle(&self, stream: TcpStream, req: Request<()>) -> std::io::Result<Option<TcpStream>>;
}

#[async_trait]
pub trait Half<Codec> {
    async fn handle(&self, ctx: &mut Context<Codec>) -> std::io::Result<()>;
}

pub struct HttpEndpoint {
    handler: Box<dyn Half<Http<Body>> + Send + Sync>,
}

impl HttpEndpoint {
    pub fn new(handler: impl Half<Http<Body>> + Send + Sync + 'static) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }
}

#[async_trait]
impl Endpoint for HttpEndpoint {
    async fn handle(&self, mut stream: TcpStream, req: Request<()>) -> std::io::Result<Option<TcpStream>> {
        let mut context: Context<Http<Body>> = Context::from(&mut stream, Some(req));

        self.handler.handle(&mut context).await?;

        Ok(None)
    }
}

pub struct WsEndpoint {
    handler: Box<dyn Half<Ws> + Send + Sync>,
}

impl WsEndpoint {
    pub fn new(handler: impl Half<Ws> + Send + Sync + 'static) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }
}

#[async_trait]
impl Endpoint for WsEndpoint {
    async fn handle(&self, mut stream: TcpStream, req: Request<()>) -> std::io::Result<Option<TcpStream>> {
        let res = WsUpgrader::upgrade(&mut stream, &req).await;
        res.unwrap();

        let mut context = Context::<Ws>::from(&mut stream, None);

        self.handler.handle(&mut context).await?;

        Ok(None)
    }
}

