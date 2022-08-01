use crate::codec::prelude::*;
use crate::context::{Body, Context};
use crate::routing::{Endpoint, Router};
use crate::ws::WsUpgrader;
use bytes::BytesMut;
use http::{Request, Response, StatusCode};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
// use tracing::info;
// use tracing_subscriber;

pub fn serve<F>(addr: &'static str, router_fn: F)
where
    F: Fn() -> Router,
{
    let addr = match addr.parse() {
        Ok(addr) => addr,
        Err(_) => {
            log::error!("failed to parse host address");
            return;
        }
    };

    log::info!("server started on http://{:?}", addr);

    let mut handles = Vec::new();

    for _ in 0..num_cpus::get() {
        let instance = router_fn();
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

async fn worker(addr: SocketAddr, router: Router) -> Result<(), Box<dyn std::error::Error>> {
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
    let router = Arc::new(router);

    loop {
        let (mut socket, addr) = incoming.accept().await?;

        log::debug!("new connection {:?}", addr);

        let instance = router.clone();

        tokio::spawn(async move {
            if let Err(e) = process(instance, &mut socket).await {
                if e.kind() == std::io::ErrorKind::ConnectionReset {
                    return;
                }

                let resp = Response::builder()
                    .status(StatusCode::REQUEST_TIMEOUT)
                    .body(().into())
                    .unwrap();

                let mut ctx: Context<Http<_>> = Context::from(&mut socket);
                let _ = ctx.send(resp).await;
                log::error!("runtime error {:?}", e);
            }
        });
    }
}

async fn process(router: Arc<Router>, stream: &mut TcpStream) -> std::io::Result<()> {
    let mut bytes = BytesMut::new();
    stream.set_nodelay(true).unwrap();

    loop {
        let mut codec = Http::new();
        if stream.read_buf(&mut bytes).await? == 0 {
            return Ok(());
        }

        let mut res = codec.decode(&mut bytes);

        while let Ok(None) = res {
            if stream.read_buf(&mut bytes).await? == 0 {
                return Ok(());
            }

            res = codec.decode(&mut bytes);
        }

        let req: Request<Body> = match res {
            Ok(Some(req)) => req,
            _ => {
                log::error!("failed to parse request bytes");
                return Ok(());
            }
        };

        let (r, params) = router.route(&req);

        let mut close = false;

        if let Some(v) = req.headers().get("Connection") {
            if v == "close" {
                close = true;
            }
        }

        match &*r {
            Endpoint::Http(r) => {
                let mut context: Context<Http<_>> = Context::from(stream);
                let resp = r.handle(&req, &params).await?;
                context.send(resp).await?;
            }
            Endpoint::Ws(r) => {
                WsUpgrader::upgrade(stream, &req).await?;

                let mut context = Context::<Ws>::from(stream);
                let (mut tx, mut rx) = context.split();
                r.handle(&mut tx, &mut rx, &params).await?;

                let close = WsFrame::builder().close();
                tx.write(close).await?;

                break;
            }
        }

        if close {
            break;
        }
    }

    Ok(())
}
