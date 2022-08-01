#![feature(io_error_other)]

pub mod codec;
pub mod context;
pub mod flow;
pub mod routing;
mod ws;

pub mod prelude {
    pub use crate::codec::prelude::*;
    pub use crate::context::{self, Body, Context};
    pub use crate::flow::serve;
    pub use crate::routing::{wrap, HttpRoute, Route, Router};
    pub use http::{Method, Request, Response, StatusCode};
    pub use trie_rs::path::params::Params;
}
