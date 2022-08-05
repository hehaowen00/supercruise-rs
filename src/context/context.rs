use crate::codec::{
    http::Http,
    websocket::{Ws, WsFrame},
    Decoder, Encoder,
};
use crate::context::Body;
use bytes::BytesMut;
use http::{Request, Response};
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

    pub fn split(&'a mut self) -> (Sender<'a, Codec>, Receiver<'a, Codec>) {
        let (reader, writer) = self.stream.split();
        let tx = Sender::new(writer);
        let rx = Receiver::new(reader);
        (tx, rx)
    }
    pub fn split_stream(stream: &'a mut TcpStream) -> (Sender<'a, Codec>, Receiver<'a, Codec>) {
        let (reader, writer) = stream.split();
        let tx = Sender::new(writer);
        let rx = Receiver::new(reader);
        (tx, rx)
    }
}

impl<'a> Context<'a, Ws> {
    pub async fn next(&mut self) -> std::io::Result<WsFrame> {
        let mut ws = Ws::new();

        let bytes = &mut self.buffers[0];
        if self.stream.read_buf(bytes).await? == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));
        }

        let mut res = ws.decode(bytes);

        while let Ok(None) = res {
            if self.stream.read_buf(bytes).await? == 0 {
                return Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));
            }
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
            buf: BytesMut::with_capacity(8192),
            _marker: PhantomData,
        }
    }

    pub fn to<Codec2>(self) -> Sender<'a, Codec2> {
        Sender {
            writer: self.writer,
            buf: self.buf,
            _marker: PhantomData,
        }
    }
}

impl<'a> Sender<'a, Ws> {
    pub async fn send(&mut self, msg: WsFrame) -> std::io::Result<()> {
        let mut ws = Ws::new();
        ws.encode(msg, &mut self.buf).unwrap();

        let res = self.writer.write_all(&self.buf).await;
        self.buf.clear();
        res
    }
}

impl<'a> Sender<'a, Http<Body>> {
    pub async fn send(&mut self, msg: Response<Body>) -> std::io::Result<()> {
        let mut http = Http::<Body>::new();

        let mut bytes = BytesMut::new();
        http.encode(msg, &mut bytes).unwrap();

        self.writer.write_all(&bytes).await
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
            buf: BytesMut::with_capacity(8192),
            _marker: PhantomData,
        }
    }

    pub fn to<Codec2>(self) -> Receiver<'a, Codec2> {
        Receiver {
            reader: self.reader,
            buf: self.buf,
            _marker: PhantomData,
        }
    }
}

impl<'a> Receiver<'a, Http<Body>> {
    pub async fn next(&mut self) -> std::io::Result<Request<Body>> {
        let mut http = Http::new();

        if self.reader.read_buf(&mut self.buf).await? == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));
        }

        let mut res = http.decode(&mut self.buf);

        while let Ok(None) = res {
            if self.reader.read_buf(&mut self.buf).await? == 0 {
                return Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));
            }
            res = http.decode(&mut self.buf);
        }

        self.buf.clear();

        match res.unwrap() {
            Some(msg) => return Ok(msg),
            _ => unreachable!(),
        }
    }
}

impl<'a> Receiver<'a, Ws> {
    pub async fn next(&mut self) -> std::io::Result<WsFrame> {
        let mut ws = Ws::new();

        if self.reader.read_buf(&mut self.buf).await? == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));
        }

        let mut res = ws.decode(&mut self.buf);

        while let Ok(None) = res {
            if self.reader.read_buf(&mut self.buf).await? == 0 {
                return Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));
            }
            res = ws.decode(&mut self.buf);
        }

        self.buf.clear();

        match res.unwrap() {
            Some(msg) => return Ok(msg),
            _ => unreachable!(),
        }
    }
}
