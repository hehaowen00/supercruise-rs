use crate::codec::{Encoder, Decoder};
use crate::codec::http::*;
use crate::codec::websocket::*;
use bytes::BytesMut;
use http::{Request, Response};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use std::sync::Arc;
use std::marker::PhantomData;
use crate::Router;

pub async fn process(router: Arc<Router>, stream: TcpStream) -> std::io::Result<()> {
    let mut bytes = BytesMut::new();

    let mut cont = Some(stream);

    loop {
        if let None = cont {
            break
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

        let req: Request<()> = match res {
            Ok(Some(req)) => req,
            _ => return Ok(()), 
        };

        match router.route(req.method(), req.uri().path()).await {
            Some(endpoint) => match endpoint.handle(stream, req).await? {
                Some(stream) => {
                    cont = Some(stream)
                },
                None => {
                }
            },
            None => {
                break
            }
        }
    }

    Ok(())
}

pub struct Context<'a, Codec> {
    stream: &'a mut TcpStream,
    req: Option<Request<()>>,
    buffers: [BytesMut; 2],
    _marker: PhantomData<Codec>,
}

pub struct Sender<'a, Codec> {
    writer: WriteHalf<'a>,
    _marker: PhantomData<Codec>,
}

impl<'a, Codec> Sender<'a, Codec> {
    pub fn new(writer: WriteHalf<'a>) -> Self {
        Self {
            writer,
            _marker: PhantomData,
        }
    }
}

pub struct Receiver<'a, Codec> {
    reader: ReadHalf<'a>,
    buf: BytesMut,
    _marker: PhantomData<Codec>,
}

impl<'a, Codec> Receiver<'a, Codec> {
    pub fn new(reader: ReadHalf<'a>) -> Self {
        Self {
            reader,
            buf: BytesMut::new(),
            _marker: PhantomData,
        }
    }

    pub async fn next(&mut self) -> std::io::Result<WsFrame> {
        let mut ws = Ws::new();

        self.reader.read_buf(&mut self.buf).await?;

        let mut res = ws.decode(&mut self.buf);

        while let Ok(None) = res {
            self.reader.read_buf(&mut self.buf).await?;
            res = ws.decode(&mut self.buf);
        }

        self.buf.clear();

        match res.unwrap() {
            Some(msg) => return Ok(msg),
            _ => unreachable!(),
        }
    }
}

impl<'a, Codec> Context<'a, Codec> {
    pub fn from(stream: &'a mut TcpStream, req: Option<Request<()>>) -> Self {
        Self {
            stream,
            req,
            buffers: [BytesMut::new(), BytesMut::new()],
            _marker: PhantomData,
        }
    }
}

impl<'a> Context<'a, Ws> {
    pub fn split(stream: &'a mut TcpStream) -> (Sender<'a, Ws>, Receiver<'a, Ws>) {
        let (reader, writer) = stream.split();
        let tx = Sender::new(writer);
        let rx = Receiver::new(reader);
        return (tx, rx)
    }

    pub async fn next(&mut self) -> std::io::Result<WsFrame> {
        let mut ws = Ws::new();

        let bytes = &mut self.buffers[0];
        self.stream.read_buf(bytes).await?;

        let mut res = ws.decode(bytes);

        while let Ok(None) = res {
            self.stream.read_buf(bytes).await?;
            res = ws.decode(bytes);
        }

        bytes.clear();

        match res.unwrap() {
            Some(msg) => return Ok(msg),
            _ => unreachable!(),
        }
    }

    pub async fn send(&mut self, msg: WsFrame) -> std::io::Result<()> {
        let mut ws = Ws::new();
        let bytes = &mut self.buffers[1];
        ws.encode(msg, bytes).unwrap();
        self.stream.write_all(bytes).await
    }
}

use crate::context::Body;

impl<'a> Context<'a, Http<Body>> {
    pub async fn send(&mut self, resp: Response<Body>) -> std::io::Result<()> {
        let mut http = Http::<Body>::new();

        let mut bytes = BytesMut::new();
        http.encode(resp, &mut bytes).unwrap();

        self.stream.write_all(&bytes).await
    }

    pub fn next(&mut self) -> Option<Request<()>> {
        self.req.take()
    }
}

