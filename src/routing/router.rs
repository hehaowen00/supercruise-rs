use crate::codec::websocket::Ws;
use crate::context::Body;
use crate::routing::route::{HttpRoute, Route};
use async_trait::async_trait;
use http::{Method, Request, Response};
use std::sync::Arc;
use trie_rs::params::Params;
use trie_rs::path::PathTrie;

pub struct Router {
    get_routes: PathTrie<Arc<Endpoint>>,
    post_routes: PathTrie<Arc<Endpoint>>,
    put_routes: PathTrie<Arc<Endpoint>>,
    delete_routes: PathTrie<Arc<Endpoint>>,
    ws: PathTrie<Arc<Endpoint>>,
    not_found: Arc<Endpoint>,
}

pub(crate) enum Endpoint {
    Http(Box<dyn HttpRoute + Send + Sync>),
    Ws(Box<dyn Route<Ws> + Send + Sync>),
}

impl Router {
    pub fn new() -> Self {
        Self {
            get_routes: PathTrie::new(),
            post_routes: PathTrie::new(),
            put_routes: PathTrie::new(),
            delete_routes: PathTrie::new(),
            ws: PathTrie::new(),
            not_found: Arc::new(Endpoint::Http(Box::new(NotFound {}))),
        }
    }

    pub fn get<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        self.get_routes
            .insert(path, Arc::new(Endpoint::Http(Box::new(route))));
        self
    }

    pub fn post<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        self.post_routes
            .insert(path, Arc::new(Endpoint::Http(Box::new(route))));
        self
    }

    pub fn put<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        self.put_routes
            .insert(path, Arc::new(Endpoint::Http(Box::new(route))));
        self
    }

    pub fn delete<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        self.delete_routes
            .insert(path, Arc::new(Endpoint::Http(Box::new(route))));
        self
    }

    pub fn ws<R>(mut self, path: &str, route: R) -> Self
    where
        R: Route<Ws> + Send + Sync + 'static,
    {
        self.ws
            .insert(path, Arc::new(Endpoint::Ws(Box::new(route))));
        self
    }

    pub fn not_found<R>(mut self, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        self.not_found = Arc::new(Endpoint::Http(Box::new(route)));
        self
    }

    #[inline]
    pub(crate) fn route<'a, 'b>(
        &'a self,
        req: &'b Request<Body>,
    ) -> (Arc<Endpoint>, Params<'a, 'b>) {
        let path = req.uri().path();
        let method = req.method();

        if let Some(value) = req.headers().get("Upgrade") {
            if value == "websocket" {
                return match self.ws.get(path) {
                    Some((r, params)) => (r.clone(), params),
                    None => (self.not_found.clone(), Params::new()),
                };
            }
        }

        match method {
            &Method::GET => match self.get_routes.get(path) {
                Some((r, params)) => (r.clone(), params),
                None => (self.not_found.clone(), Params::new()),
            },
            &Method::POST => match self.post_routes.get(path) {
                Some((r, params)) => (r.clone(), params),
                None => (self.not_found.clone(), Params::new()),
            },
            &Method::PUT => match self.put_routes.get(path) {
                Some((r, params)) => (r.clone(), params),
                None => (self.not_found.clone(), Params::new()),
            },
            &Method::DELETE => match self.delete_routes.get(path) {
                Some((r, params)) => (r.clone(), params),
                None => (self.not_found.clone(), Params::new()),
            },
            _ => (self.not_found.clone(), Params::new()),
        }
    }
}

struct NotFound;

#[async_trait]
impl HttpRoute for NotFound {
    async fn handle(
        &self,
        req: &Request<Body>,
        _params: &Params,
    ) -> std::io::Result<Response<Body>> {
        let resp: Response<Body> = Response::builder()
            .status(404)
            .header("Content-Type", "text/html")
            .body(String::from("404 Not Found").into())
            .unwrap();

        log::warn!("404 Not Found {}", req.uri().path());

        Ok(resp)
    }
}
