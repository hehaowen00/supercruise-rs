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

pub(crate) async fn process(router: Arc<Router>, mut stream: TcpStream) -> std::io::Result<()> {
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

        let (r, params) = router.route(req.method(), req.uri().path());

        match &*r {
            Endpoint::Http(r) => {
                let mut context: Context<Http<_>> = Context::from(&mut stream);
                let resp = r.handle(&req, params).await?;

                context.send(resp).await?;
            }
            Endpoint::Ws(r) => {
                WsUpgrader::upgrade(&mut stream, &req).await?;

                let mut context = Context::<Ws>::from(&mut stream);
                r.handle(&mut context, params).await?;

                let close = WsFrame::builder().close();
                context.send(close).await?;

                break;
            }
        }
    }

    Ok(())
}
