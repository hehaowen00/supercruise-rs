mod codec;
pub(crate) mod http;
pub(crate) mod websocket;

pub(crate) use codec::{Decoder, Encoder};

pub(crate) mod prelude {
    pub use super::codec::{Decoder, Encoder};
    pub use super::http::*;
    pub use super::websocket::*;
}

#[cfg(test)]
mod http_test;
