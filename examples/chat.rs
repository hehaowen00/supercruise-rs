use async_trait::async_trait;
use once_cell::sync::Lazy;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use supercruise_rs::{
    prelude::{context, *},
    routing::FnOutput,
};
use tokio::sync::broadcast;

static CHAT: Lazy<ChatHandle> = Lazy::new(|| ChatHandle::new());
static HTML: &'static str = include_str!("chat.html");

struct Chat {
    id: AtomicUsize,
    tx: broadcast::Sender<(usize, Vec<u8>)>,
    _rx: broadcast::Receiver<(usize, Vec<u8>)>,
}

impl Chat {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(64);
        Self {
            id: AtomicUsize::new(0),
            tx,
            _rx,
        }
    }
}

#[async_trait]
impl Route<Ws> for Chat {
    async fn handle(
        &self,
        tx: &mut context::Sender<Ws>,
        rx: &mut context::Receiver<Ws>,
        _params: &Params,
    ) -> std::io::Result<()> {
        let mut chat_rx = self.tx.subscribe();
        let chat_tx = self.tx.clone();
        let chat_id = self.id.fetch_add(1, Ordering::SeqCst);

        loop {
            tokio::select! {
                evt = rx.next() => if let Ok(frame) = evt {
                    match frame.opcode() {
                        Opcode::TEXT => {
                            chat_tx.send((chat_id, frame.data().to_vec())).unwrap();
                        }
                        Opcode::PING => {
                            tx.write(WsFrame::builder().pong()).await?;
                        }
                        Opcode::PONG => {
                            tx.write(WsFrame::builder().ping()).await?;
                        }
                        Opcode::CLOSE => break,
                        _ => {}
                    }
                },
                evt = chat_rx.recv() => {
                    if let Ok((_, data)) = evt {
                        let frame = WsFrame::builder().text(data);
                        tx.write(frame).await?;
                    }
                }
            }
        }

        Ok(())
    }
}

struct ChatHandle {
    inner: Arc<Chat>,
}

impl ChatHandle {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Chat::new()),
        }
    }
}

impl Clone for ChatHandle {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[async_trait]
impl Route<Ws> for ChatHandle {
    #[inline]
    async fn handle(
        &self,
        tx: &mut context::Sender<Ws>,
        rx: &mut context::Receiver<Ws>,
        params: &Params,
    ) -> std::io::Result<()> {
        self.inner.handle(tx, rx, params).await
    }
}

fn index(_req: &Request<Body>, _params: &Params) -> FnOutput<Response<Body>> {
    Box::pin(async {
        let resp: Response<Body> = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .header("Connection", "keep-alive")
            .body(HTML.into())
            .unwrap();

        Ok(resp)
    })
}

fn make_router() -> Router {
    Router::builder()
        .get("/", wrap(index))
        .ws("/chat", CHAT.clone())
        .finalize()
}

fn main() {
    env_logger::init();
    serve("0.0.0.0:8080", make_router);
}
