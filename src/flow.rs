use crate::codec::prelude::*;
use crate::context::Body;
use crate::context::Context;
use crate::router::EndpointT;
use crate::Router;
use bytes::BytesMut;
use http::Request;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::ws::WsUpgrader;

pub async fn process(router: Arc<Router>, mut stream: TcpStream) -> std::io::Result<()> {
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
            _ => return Ok(()),
        };

        match router.route(req.method(), req.uri().path()).await {
            Some(e) => match &*e {
                EndpointT::Http(r) => {
                    let mut context: Context<Http<_>> = Context::from(&mut stream);

                    let resp = r.handle(req).await?;

                    context.send(resp).await?;
                }
                EndpointT::Ws(r) => {
                    WsUpgrader::upgrade(&mut stream, req).await?;

                    let mut context = Context::<Ws>::from(&mut stream);

                    r.handle(&mut context).await?;

                    let close = WsFrame::builder().close();

                    context.send(close).await?;

                    break;
                }
            },
            None => unreachable!(),
        }
    }

    Ok(())
}
