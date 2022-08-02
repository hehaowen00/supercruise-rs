pub mod codec;
pub mod context;
pub mod routing;
pub mod worker;
mod ws;

pub mod prelude {
    pub use crate::codec::{
        http::Http,
        websocket::{Opcode, Ws, WsFrame, WsFrameBuilder},
    };
    pub use crate::context::{self, Body, Context};
    pub use crate::routing::{wrap, HttpRoute, Route, Router};
    pub use crate::worker::serve;
    pub use http::{Method, Request, Response, StatusCode};
    pub use trie_rs::path::params::Params;
}
