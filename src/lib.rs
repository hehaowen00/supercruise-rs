// #![allow(unused)]
#![feature(io_error_other)]
pub mod codec;
pub mod context;
pub mod flow;
pub mod route;
pub mod router;
mod ws;

pub mod prelude {
    pub use super::codec::prelude::*;
    pub use super::context::{self, Body, Context};
    pub use super::route::{HttpRoute, Route};
    pub use super::router::Router;
    pub use http::{Method, Request, Response, StatusCode};
}

use crate::codec::prelude::*;
use crate::context::Context;
pub use http::Method;
use http::{Response, StatusCode};
use router::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
// use tracing::info;
// use tracing_subscriber;

async fn worker(addr: SocketAddr, router: Arc<Router>) -> Result<(), Box<dyn std::error::Error>> {
    let socket = socket2::Socket::new(
        match addr {
            SocketAddr::V4(_) => socket2::Domain::IPV4,
            SocketAddr::V6(_) => socket2::Domain::IPV6,
        },
        socket2::Type::STREAM,
        None,
    )
    .unwrap();

    socket.set_reuse_address(true).unwrap();
    socket.set_reuse_port(true).unwrap();
    socket.set_nonblocking(true).unwrap();
    socket.bind(&addr.into()).unwrap();
    socket.listen(0).unwrap();

    let incoming = TcpListener::from_std(socket.into()).unwrap();

    loop {
        let (mut socket, addr) = incoming.accept().await?;

        log::debug!("new connection {:?}", addr);

        let r_clone = router.clone();

        tokio::spawn(async move {
            if let Err(e) = flow::process(r_clone, &mut socket).await {
                if e.kind() == std::io::ErrorKind::ConnectionReset {
                    return;
                }

                let resp = Response::builder()
                    .status(StatusCode::REQUEST_TIMEOUT)
                    .body(().into())
                    .unwrap();

                let mut ctx: Context<Http<_>> = Context::from(&mut socket);
                let _ = ctx.send(resp).await;
                log::error!("{:?}", e);
            }
        });
    }
}

pub fn serve(addr: &'static str, router: Router) {
    let addr = match addr.parse() {
        Ok(addr) => addr,
        Err(_) => {
            log::error!("failed to parse host address");
            return;
        }
    };
    let router = Arc::new(router);

    log::info!("server started on http://{:?}", addr);

    let mut handles = Vec::new();

    for _ in 0..num_cpus::get() {
        let instance = router.clone();
        let h = std::thread::spawn(move || {
            let res = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(worker(addr, instance));

            log::error!("runtime exited: {:?}", res);
        });

        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }
}
