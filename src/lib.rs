#![feature(io_error_other)]

pub mod flow;
mod ws;
pub mod route;
pub mod context;
pub mod common;
pub mod router;

pub mod codec;
use crate::codec::http::Http;
use std::sync::Arc;
use tokio::net::TcpListener;
use async_trait::async_trait;
use crate::route::{Endpoint, WsEndpoint, HttpEndpoint};

pub use http::Method;

pub async fn serve(router: Router) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";

    let listener = TcpListener::bind(&addr).await?;

    println!("started: http://{}", addr);

    let router = Arc::new(router);

    loop {
        let (socket, addr) = listener.accept().await?;

        println!("new connection {:?}", addr);

        let r_clone = router.clone();

        tokio::spawn(async move {
            if let Err(e) = flow::process(r_clone, socket).await {
                println!("[error] {:?}", e);
            }
        });
    }
}

pub struct Router {
    http: Option<Arc<Box<dyn Endpoint>>>,
    ws: Option<Arc<Box<dyn Endpoint>>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            http: None,
            ws: None,
        }
    }

    pub fn get<R>(mut self, path: &str, route: R) -> Self
    where
        R: Half<Http<Body>> + Send + Sync + 'static,
    {
        let route: Arc<Box<dyn Endpoint>> = Arc::new(Box::new(HttpEndpoint::new(route)));
        self.http = Some(route);
        self
    }

    pub fn post<R>(mut self, path: &str, route: R) -> Self
    where
        R: Half<Http<Body>> + Send + Sync + 'static,
    {
        let route: Arc<Box<dyn Endpoint>> = Arc::new(Box::new(HttpEndpoint::new(route)));
        self
    }

    pub fn put<R>(mut self, path: &str, route: R) -> Self
    where
        R: Half<Http<Body>> + Send + Sync + 'static,
    {
        let route: Arc<Box<dyn Endpoint>> = Arc::new(Box::new(HttpEndpoint::new(route)));
        self
    }

    pub fn delete<R>(mut self, path: &str, route: R) -> Self
    where
        R: Half<Http<Body>> + Send + Sync + 'static,
    {
        let route: Arc<Box<dyn Endpoint>> = Arc::new(Box::new(HttpEndpoint::new(route)));
        self
    }

    pub fn ws<R>(mut self, path: &str, route: R) -> Self
    where
        R: Half<Ws> + Send + Sync + 'static,
    {
        let route: Arc<Box<dyn Endpoint>> = Arc::new(Box::new(WsEndpoint::new(route)));
        self.ws = Some(route);
        self
    }

    pub fn not_found(mut self) -> Self {
        self
    }

    pub async fn route(&self, method: &Method, path: &str) -> Option<Arc<Box<dyn route::Endpoint>>> {
        if path == "/" {
            return self.http.clone();
        } else if path == "/ws" {
            return self.ws.clone();
        }

        let e = HttpEndpoint::new(NotFound{});
        Some(Arc::new(Box::new(e)))
    }
}

use crate::context::Body;
use crate::flow::Context;
use http::Response;
use crate::route::Half;
use crate::codec::websocket::Ws;

struct NotFound;

#[async_trait]
impl Half<Http<Body>> for NotFound {
    async fn handle(&self, ctx: &mut Context<Http<Body>>) -> std::io::Result<()> {
        let resp: Response<Body> = Response::builder()
            .header("Content-Type", "text/html")
            .body(String::from("404 Not Found").into()).unwrap();

        ctx.send(resp).await?;

        println!("404 Not Found {}", ctx.next().unwrap().uri().path());

        Ok(())
    }
}

