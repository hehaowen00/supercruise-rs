use crate::codec::prelude::*;
use crate::context::Body;
use crate::Router;
use bytes::BytesMut;
use http::Request;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

pub async fn process(router: Arc<Router>, stream: TcpStream) -> std::io::Result<()> {
    let mut bytes = BytesMut::new();

    let mut cont = Some(stream);

    loop {
        if let None = cont {
            break;
        }
        let mut stream = cont.take().unwrap();

        stream.set_nodelay(true).unwrap();

        stream.read_buf(&mut bytes).await?;

        let mut codec = Http::new();

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
            Some(endpoint) => match endpoint.handle(stream, req).await? {
                Some(stream) => cont = Some(stream),
                None => {}
            },
            None => break,
        }
    }

    Ok(())
}
