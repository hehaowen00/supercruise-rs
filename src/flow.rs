use crate::codec::prelude::*;
use crate::context::{Body, Context};
use crate::router::Endpoint;
use crate::ws::WsUpgrader;
use crate::Router;
use bytes::BytesMut;
use http::Request;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

pub(crate) async fn process(router: Arc<Router>, stream: &mut TcpStream) -> std::io::Result<()> {
    let mut bytes = BytesMut::new();
    stream.set_nodelay(true).unwrap();

    loop {
        let mut codec = Http::new();

        stream.read_buf(&mut bytes).await?;

        let mut res = codec.decode(&mut bytes);

        while let Ok(None) = res {
            stream.read_buf(&mut bytes).await?;
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
                log::debug!("http endpoint");
                let mut context: Context<Http<_>> = Context::from(stream);
                let resp = r.handle(req).await?;
                context.send(resp).await?;
            }
            Endpoint::Ws(r) => {
                log::debug!("websocket endpoint");
                WsUpgrader::upgrade(stream, &req).await?;
                let mut context = Context::<Ws>::from(stream);

                let (mut tx, mut rx) = context.split();
                r.handle(&mut tx, &mut rx).await?;

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
