use crate::prelude::*;
use async_trait::async_trait;
use http::Response;
use tokio::io::AsyncReadExt;

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

pub struct File {
    path: std::path::PathBuf,
}

impl File {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

#[async_trait]
impl Route<Http<Body>> for File {
    async fn handle(&self, ctx: &mut Context<Http<Body>>) -> std::io::Result<()> {
        let mut f = tokio::fs::File::open(&self.path).await.unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).await?;

        let resp: Response<Body> = Response::builder()
            .header("Content-Type", "text/html")
            .header("Connection", "close")
            .body(buf.clone().into())
            .unwrap();

        ctx.send(resp).await?;

        Ok(())
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

#[async_trait]
impl Route<Http<Body>> for Redirect {
    async fn handle(&self, ctx: &mut Context<Http<Body>>) -> std::io::Result<()> {
        Ok(())
    }
}
