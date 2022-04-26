#![feature(io_error_other)]

pub mod codec;
pub mod common;
pub mod context;
pub mod flow;
pub mod route;
pub mod router;
mod ws;

pub mod prelude {
    pub use super::codec::prelude::*;
    pub use super::context::{Body, Context};
    pub use super::route::{HttpRoute, Route};
    pub use super::router::Router;
    pub use http::{Method, Request, Response, StatusCode};
}

use router::Router;
use std::sync::Arc;
use tokio::net::TcpListener;

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
