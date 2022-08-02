mod codec;

pub(crate) mod http;
pub(crate) mod websocket;
pub(crate) use codec::{Decoder, Encoder};

#[cfg(test)]
mod http_test;
