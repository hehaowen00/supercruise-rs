pub(crate) mod node;
pub mod params;
#[cfg(test)]
mod test;

use crate::codec::websocket::Ws;
use crate::route::{Endpoint, HttpEndpoint, HttpRoute, Route, WsEndpoint};
use http::{Method, Request, Response};
use std::sync::Arc;

pub enum EndpointR {
    Http(Box<dyn HttpRoute + Send + Sync>),
    Ws(Box<dyn Route<Ws> + Send + Sync>),
}

pub struct TrieRouter {
    get_routes: (),
    post_routes: (),
    put_routes: (),
    delete_routes: (),
    ws: (),
    not_found: (),
}

impl TrieRouter {
    pub fn new() -> Self {
        Self {
            get_routes: (),
            post_routes: (),
            put_routes: (),
            delete_routes: (),
            ws: (),
            not_found: (),
        }
    }

    pub fn insert<R>(&mut self, path: &str, route: R)
    where
        R: HttpRoute + Send + Sync + 'static,
    {
    }

    pub fn get<R>(&mut self, method: Method, path: &str) -> Option<Arc<EndpointR>>
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        None
    }
}
