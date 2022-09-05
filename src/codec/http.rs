use crate::codec::{Decoder, Encoder};
use crate::context::Body;
use bytes::BytesMut;
use http::header::HeaderValue;
use http::{Request, Response};
use std::fmt::{self, Write};
use std::marker::PhantomData;

pub struct Http<T> {
    _marker: PhantomData<T>,
}

impl<T> Http<T> {
    pub fn new() -> Self {
        Http {
            _marker: PhantomData,
        }
    }
}

impl Decoder for Http<Body> {
    type Item = Request<Body>;
    type Error = ();

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() == 0 {
            return Ok(None);
        }

        let mut headers = [None; 32];

        let (method, path, version, amt) = {
            let mut parsed = [httparse::EMPTY_HEADER; 32];
            let mut r = httparse::Request::new(&mut parsed);

            let amt = match r.parse(src) {
                Ok(httparse::Status::Complete(amt)) => amt,
                Ok(httparse::Status::Partial) => return Ok(None),
                e => {
                    log::error!("codec http amt {:?}", e);
                    return Err(());
                }
            };

            for (i, header) in r.headers.iter().enumerate() {
                let k = header.name;
                let v = header.value;
                headers[i] = Some((k, v));
            }
            (r.method.unwrap(), r.path.unwrap(), r.version.unwrap(), amt)
        };

        if version != 1 {
            return Err(());
        }

        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .version(http::Version::HTTP_11);

        for header in headers.iter() {
            let (k, v) = match *header {
                Some((k, v)) => (k, v),
                None => break,
            };

            let value = HeaderValue::from_bytes(v).unwrap();
            builder = builder.header(k, value);
        }

        let _ = src.split_to(amt);
        let req = builder.body(Body::from(src)).unwrap();

        src.clear();

        Ok(Some(req))
    }
}

impl<T> Encoder<Response<()>> for Http<T> {
    type Error = ();

    fn encode(&mut self, item: Response<()>, dest: &mut BytesMut) -> Result<(), Self::Error> {
        write!(ByteWriter(dest), "HTTP/1.1 {}\r\n", item.status(),).unwrap();

        for (k, v) in item.headers() {
            dest.extend_from_slice(k.as_str().as_bytes());
            dest.extend_from_slice(b": ");
            dest.extend_from_slice(v.as_bytes());
            dest.extend_from_slice(b"\r\n");
        }

        dest.extend_from_slice(b"\r\n");

        Ok(())
    }
}

impl Encoder<Response<Body>> for Http<Body> {
    type Error = ();

    fn encode(&mut self, item: Response<Body>, dest: &mut BytesMut) -> Result<(), Self::Error> {
        write!(
            ByteWriter(dest),
            "HTTP/1.1 {}\r\ncontent-length: {}\r\n",
            item.status(),
            item.body().len(),
        )
        .unwrap();

        for (k, v) in item.headers() {
            dest.extend_from_slice(k.as_str().as_bytes());
            dest.extend_from_slice(b": ");
            dest.extend_from_slice(v.as_bytes());
            dest.extend_from_slice(b"\r\n");
        }

        dest.extend_from_slice(b"\r\n");
        item.body().bytes(dest);

        Ok(())
    }
}

struct ByteWriter<'a>(&'a mut BytesMut);

impl fmt::Write for ByteWriter<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }

    fn write_fmt(self: &mut Self, args: std::fmt::Arguments<'_>) -> std::fmt::Result {
        fmt::write(self, args)
    }
}
