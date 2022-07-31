use std::sync::atomic::AtomicBool;

use crate::prelude::*;
use async_trait::async_trait;
use bytes::BytesMut;
use http::Response;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use trie_rs::path::params::Params;

pub struct Dir {
    path: std::path::PathBuf,
}

impl Dir {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

#[async_trait]
impl HttpRoute for Dir {
    async fn handle(&self, req: Request<Body>) -> std::io::Result<Response<Body>> {
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(String::from("ok").into())
            .unwrap();

        Ok(resp)
    }
}

pub struct File {
    path: std::path::PathBuf,
    empty: AtomicBool,
    contents: tokio::sync::RwLock<Vec<u8>>,
}

impl File {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            empty: AtomicBool::new(true),
            contents: RwLock::new(Vec::new()),
        }
    }

    pub async fn serve(&self, dest: &mut Vec<u8>) -> std::io::Result<()> {
        match self.empty.load(std::sync::atomic::Ordering::SeqCst) {
            true => {
                let mut guard = self.contents.write().await;
                println!("read");
                let mut f = tokio::fs::File::open(&self.path).await?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf).await?;
                dest.extend_from_slice(&buf);

                *guard = buf;
                drop(guard);
                self.empty.store(false, std::sync::atomic::Ordering::SeqCst);
            }
            false => {
                let guard = self.contents.read().await;
                dest.extend_from_slice(&guard);
                drop(guard)
            }
        }
        Ok(())
    }
}

#[async_trait]
impl HttpRoute for File {
    async fn handle(&self, req: Request<Body>) -> std::io::Result<Response<Body>> {
        let mut buf = Vec::new();
        self.serve(&mut buf).await?;

        let resp: Response<Body> = Response::builder()
            .header("Content-Type", "text/html")
            .header("Connection", "close")
            .body(buf.into())
            .unwrap();

        Ok(resp)
    }
}

pub struct Redirect {
    path: String,
}

impl Redirect {
    pub fn to<S>(path: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            path: path.as_ref().to_string(),
        }
    }
}

// #[async_trait]
// impl Route<Http<Body>> for Redirect {
//     async fn handle(&self, ctx: Context<Http<Body>>) -> std::io::Result<()> {
//         Ok(())
//     }
// }
