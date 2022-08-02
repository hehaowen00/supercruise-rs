use crate::codec::websocket::Ws;
use crate::context::Body;
use crate::routing::route::{HttpRoute, Route};
use async_trait::async_trait;
use http::{Method, Request, Response};
use std::sync::Arc;
use trie_rs::path::{Params, PathTrie, TrieBuilder};

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
    pub fn builder() -> RouterBuilder {
        RouterBuilder::new()
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

pub struct RouterBuilder {
    get_routes: TrieBuilder<Arc<Endpoint>>,
    post_routes: TrieBuilder<Arc<Endpoint>>,
    put_routes: TrieBuilder<Arc<Endpoint>>,
    delete_routes: TrieBuilder<Arc<Endpoint>>,
    ws: TrieBuilder<Arc<Endpoint>>,
    not_found: Option<Arc<Endpoint>>,
}

impl RouterBuilder {
    pub fn new() -> Self {
        Self {
            get_routes: TrieBuilder::new(),
            post_routes: TrieBuilder::new(),
            put_routes: TrieBuilder::new(),
            delete_routes: TrieBuilder::new(),
            ws: TrieBuilder::new(),
            not_found: Some(Arc::new(Endpoint::Http(Box::new(NotFound {})))),
        }
    }

    pub fn get<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        let route: Arc<Endpoint> = Arc::new(Endpoint::Http(Box::new(route)));
        self.get_routes.insert(path, route);
        self
    }

    pub fn post<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        let route: Arc<Endpoint> = Arc::new(Endpoint::Http(Box::new(route)));
        self.post_routes.insert(path, route);
        self
    }

    pub fn put<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        let route: Arc<Endpoint> = Arc::new(Endpoint::Http(Box::new(route)));
        self.put_routes.insert(path, route);
        self
    }

    pub fn delete<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        let route: Arc<Endpoint> = Arc::new(Endpoint::Http(Box::new(route)));
        self.delete_routes.insert(path, route);
        self
    }

    pub fn ws<R>(mut self, path: &str, route: R) -> Self
    where
        R: Route<Ws> + Send + Sync + 'static,
    {
        let route: Arc<Endpoint> = Arc::new(Endpoint::Ws(Box::new(route)));
        self.ws.insert(path, route);
        self
    }

    pub fn not_found<R>(mut self, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        let route: Arc<Endpoint> = Arc::new(Endpoint::Http(Box::new(route)));
        self.not_found = Some(route);
        self
    }

    pub fn finalize(self) -> Router {
        Router {
            get_routes: self.get_routes.finalize(),
            post_routes: self.post_routes.finalize(),
            put_routes: self.put_routes.finalize(),
            delete_routes: self.delete_routes.finalize(),
            ws: self.ws.finalize(),
            not_found: self.not_found.unwrap(),
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
