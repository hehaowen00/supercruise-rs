// #![allow(unused)]
#![feature(generic_associated_types)]
#![feature(io_error_other)]
pub mod codec;
pub mod context;
pub mod flow;
pub mod routing;
mod ws;

pub mod prelude {
    pub use super::codec::prelude::*;
    pub use super::context::{self, Body, Context};
    pub use super::flow::serve;
    pub use super::routing::Router;
    pub use super::routing::{HttpRoute, Route};
    pub use http::{Method, Request, Response, StatusCode};
    pub use trie_rs::path::params::Params;
}
