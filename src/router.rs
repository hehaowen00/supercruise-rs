use crate::codec::websocket::Ws;
use crate::context::Body;
use crate::route::{Endpoint, HttpEndpoint, WsEndpoint};
use crate::route::{HttpRoute, Route};
use async_trait::async_trait;
use http::{Method, Request, Response};
use std::sync::Arc;
use trie_rs::radix::RadixNode;
use trie_rs::TrieExt;

pub struct Router {
    get_routes: RadixNode<String, Arc<EndpointR>>,
    post_routes: RadixNode<String, Arc<EndpointR>>,
    put_routes: RadixNode<String, Arc<EndpointR>>,
    delete_routes: RadixNode<String, Arc<EndpointR>>,
    ws: Option<Arc<EndpointR>>,
    not_found: Option<Arc<EndpointR>>,
}

pub(crate) enum EndpointR {
    Http(Box<dyn HttpRoute + Send + Sync>),
    Ws(Box<dyn Route<Ws> + Send + Sync>),
}

impl Router {
    pub fn new() -> Self {
        let e = NotFound {};

        Self {
            ws: None,
            get_routes: RadixNode::new(),
            post_routes: RadixNode::new(),
            put_routes: RadixNode::new(),
            delete_routes: RadixNode::new(),
            not_found: Some(Arc::new(EndpointR::Http(Box::new(e)))),
        }
    }

    pub fn get<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        let route: Arc<EndpointR> = Arc::new(EndpointR::Http(Box::new(route)));
        let xs: Vec<_> = if path == "/" {
            vec![path.to_string()]
        } else {
            path.split("/")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        };
        self.get_routes.insert(&xs, route).unwrap();
        self
    }

    pub fn post<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        let route: Arc<EndpointR> = Arc::new(EndpointR::Http(Box::new(route)));
        let xs: Vec<_> = if path == "/" {
            vec![path.to_string()]
        } else {
            path.split("/")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        };

        self.post_routes.insert(&xs, route).unwrap();
        self
    }

    pub fn put<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        let route: Arc<Box<dyn Endpoint>> = Arc::new(Box::new(HttpEndpoint::new(route)));

        self
    }

    pub fn delete<R>(mut self, path: &str, route: R) -> Self
    where
        R: HttpRoute + Send + Sync + 'static,
    {
        let route: Arc<Box<dyn Endpoint>> = Arc::new(Box::new(HttpEndpoint::new(route)));
        self
    }

    pub fn ws<R>(mut self, path: &str, route: R) -> Self
    where
        R: Route<Ws> + Send + Sync + 'static,
    {
        let route: Arc<EndpointR> = Arc::new(EndpointR::Ws(Box::new(route)));

        self.ws = Some(route);
        self
    }

    pub fn not_found(mut self) -> Self {
        self
    }

    #[inline]
    pub(crate) async fn route(&self, method: &Method, path: &str) -> Option<Arc<EndpointR>> {
        let xs: Vec<_> = if path == "/" {
            vec![path.to_string()]
        } else {
            path.split("/")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        };

        if path == "/ws" {
            return self.ws.clone();
        }

        match method {
            &Method::GET => match self.get_routes.get(&xs) {
                Some(r) => Some(r.clone()),
                _ => self.not_found.clone(),
            },
            &Method::POST => match self.post_routes.get(&xs) {
                Some(r) => Some(r.clone()),
                _ => self.not_found.clone(),
            },
            _ => self.not_found.clone(),
        }
    }
}

struct NotFound;

#[async_trait]
impl HttpRoute for NotFound {
    async fn handle(&self, req: Request<Body>) -> std::io::Result<Response<Body>> {
        let resp: Response<Body> = Response::builder()
            .header("Content-Type", "text/html")
            .body(String::from("404 Not Found").into())
            .unwrap();

        println!("404 Not Found {}", req.uri().path());

        Ok(resp)
    }
}
