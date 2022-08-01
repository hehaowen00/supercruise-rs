use crate::codec::prelude::*;
use crate::context::Body;
use bytes::BytesMut;
use http::Response;
use std::marker::PhantomData;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;

pub struct Context<'a, Codec> {
    stream: &'a mut TcpStream,
    buffers: [BytesMut; 2],
    _marker: PhantomData<Codec>,
}

impl<'a, Codec> Context<'a, Codec> {
    pub fn from(stream: &'a mut TcpStream) -> Self {
        Self {
            stream,
            buffers: [BytesMut::new(), BytesMut::new()],
            _marker: PhantomData,
        }
    }
}

impl<'a> Context<'a, Ws> {
    pub fn split(&'a mut self) -> (Sender<'a, Ws>, Receiver<'a, Ws>) {
        let (reader, writer) = self.stream.split();
        let tx = Sender::new(writer);
        let rx = Receiver::new(reader);
        (tx, rx)
    }

    pub fn set_timeout(&mut self) {}

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

impl<'a> Context<'a, Http<Body>> {
    pub async fn send(&mut self, resp: Response<Body>) -> std::io::Result<()> {
        let mut http = Http::<Body>::new();

        let mut bytes = BytesMut::new();
        http.encode(resp, &mut bytes).unwrap();

        self.stream.write_all(&bytes).await
    }
}

pub struct Sender<'a, Codec> {
    writer: WriteHalf<'a>,
    buf: BytesMut,
    _marker: PhantomData<Codec>,
}

impl<'a, Codec> Sender<'a, Codec> {
    pub fn new(writer: WriteHalf<'a>) -> Self {
        Self {
            writer,
            buf: BytesMut::new(),
            _marker: PhantomData,
        }
    }

    pub async fn write(&mut self, msg: WsFrame) -> std::io::Result<()> {
        let mut ws = Ws::new();
        ws.encode(msg, &mut self.buf).unwrap();

        let res = self.writer.write_all(&self.buf).await;
        self.buf.clear();
        res
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
