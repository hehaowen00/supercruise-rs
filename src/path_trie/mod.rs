pub mod node;
pub mod params;

use http::{Method, Request, Response};
use crate::route::{Endpoint, HttpEndpoint, WsEndpoint, HttpRoute, Route};
use std::sync::Arc;

pub(crate) enum EndpointR {
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
        }
    }

    pub fn insert<R>(&mut self, path: &str, route: R)
    where
        R: HttpRoute + Send + Sync + 'static,
    {
    }

    pub fn get(&mut self, method: Method, path: &str) -> Option<Arc<EndpointR>>
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        None
    }
}
