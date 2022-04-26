use bytes::BytesMut;
use http::header::HeaderValue;
use http::{Request, Response};
use std::fmt::{self, Write};
use std::marker::PhantomData;

use crate::codec::{Decoder, Encoder};

pub struct Http<T>(PhantomData<T>);

impl<T> Http<T> {
    pub fn new() -> Self {
        Http(PhantomData)
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

pub trait FromBytes
where
    Self: Sized,
{
    fn from(bytes: &BytesMut) -> Self;
}

impl FromBytes for BytesMut {
    fn from(bytes: &BytesMut) -> Self {
        bytes.clone()
    }
}

impl FromBytes for () {
    fn from(_bytes: &BytesMut) -> () {
        ()
    }
}

use crate::context::Body;

impl Decoder for Http<Body> {
    type Item = Request<Body>;
    type Error = ();

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() == 0 {
            return Ok(None);
        }

        let mut headers = [None; 16];

        let (method, path, version, amt) = {
            let mut parsed = [httparse::EMPTY_HEADER; 16];
            let mut r = httparse::Request::new(&mut parsed);

            let amt = match r.parse(src) {
                Ok(httparse::Status::Complete(amt)) => amt,
                Ok(httparse::Status::Partial) => return Ok(None),
                _ => return Err(()),
            };

            let to_slice = |a: &[u8]| {
                let start = a.as_ptr() as usize - src.as_ptr() as usize;
                assert!(start < src.len());
                (start, start + a.len())
            };

            for (i, header) in r.headers.iter().enumerate() {
                let k = to_slice(header.name.as_bytes());
                let v = to_slice(header.value);
                headers[i] = Some((k, v));
            }

            (
                to_slice(r.method.unwrap().as_bytes()),
                to_slice(r.path.unwrap().as_bytes()),
                r.version.unwrap(),
                amt,
            )
        };

        if version != 1 {
            return Err(());
        }

        let data = src.split_to(amt).freeze();

        let s = data.slice(path.0..path.1);
        let s = unsafe { String::from_utf8_unchecked(Vec::from(s.as_ref())) };

        let mut builder = Request::builder()
            .method(&data[method.0..method.1])
            .uri(s)
            .version(http::Version::HTTP_11);

        for header in headers.iter() {
            let (k, v) = match *header {
                Some((ref k, ref v)) => (k, v),
                None => break,
            };

            let value = HeaderValue::from_bytes(&data.slice(v.0..v.1).as_ref()).unwrap();
            builder = builder.header(&data[k.0..k.1], value);
        }

        let req = builder.body(Body::from(src)).unwrap();

        src.clear();

        Ok(Some(req))
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
