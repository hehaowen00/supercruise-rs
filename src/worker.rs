use crate::codec::{
    http::Http,
    websocket::{Ws, WsFrame},
    Decoder,
};
use crate::context::{Body, Context};
use crate::routing::{Endpoint, Router};
use crate::ws::{ErrorEnum, WsUpgrader};
use bytes::BytesMut;
use http::{Request, Response, StatusCode};
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

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

    let handles: Vec<_> = (0..num_cpus::get())
        .map(|_| spawn_worker(addr, router_fn()))
        .collect();

    handles.into_iter().for_each(|h| h.join().unwrap());
}

fn spawn_worker(addr: SocketAddr, router: Router) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let res = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(worker(addr, router));

        log::error!("runtime exited: {:?}", res);
    })
}

async fn worker(
    addr: SocketAddr,
    router: Router,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
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
    socket.listen(8192).unwrap();

    let incoming = TcpListener::from_std(socket.into()).unwrap();
    let router: &'static Router = Box::leak(Box::new(router));

    loop {
        let (mut socket, addr) = incoming.accept().await?;
        let instance = router.clone();

        log::debug!("new connection {:?}", addr);

        tokio::spawn(async move {
            if let Err(e) = process(&instance, &mut socket).await {
                match e {
                    ErrorEnum::IO(ref err) => match err.kind() {
                        std::io::ErrorKind::ConnectionAborted
                        | std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::ConnectionRefused => {}
                        e => {
                            let resp = Response::builder()
                                .status(StatusCode::REQUEST_TIMEOUT)
                                .body(().into())
                                .unwrap();

                            let mut ctx = Context::<Http<_>>::from(&mut socket);
                            let _ = ctx.send(resp).await;
                            log::debug!("error {}", e);
                        }
                    },
                    _ => {}
                }
            }
        });
    }
}

async fn process(router: &'static Router, stream: &mut TcpStream) -> Result<(), ErrorEnum> {
    let mut bytes = BytesMut::with_capacity(8192);
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

        match &*r {
            Endpoint::Http(r) => {
                let mut context: Context<Http<_>> = Context::from(stream);
                let resp = r.handle(&req, &params).await?;
                if let Some(v) = resp.headers().get("Connection") {
                    if v == "close" {
                        close = true;
                    }
                }

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
