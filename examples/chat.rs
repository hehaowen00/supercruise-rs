use async_trait::async_trait;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use supercruise_rs::prelude::{context, *};
use supercruise_rs::route::Route;
use tokio::sync::broadcast::{Receiver, Sender};

static CHAT_HTML: &'static str = include_str!("chat.html");

struct Chat {
    id: AtomicUsize,
    tx: Sender<(usize, Vec<u8>)>,
    rx: Receiver<(usize, Vec<u8>)>,
}

impl Chat {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::broadcast::channel(64);
        Self {
            id: AtomicUsize::new(0),
            tx,
            rx,
        }
    }
}

#[async_trait]
impl Route<Ws> for Chat {
    async fn handle(
        &self,
        tx: &mut context::Sender<Ws>,
        rx: &mut context::Receiver<Ws>,
    ) -> std::io::Result<()> {
        let mut chat_rx = self.tx.subscribe();
        let chat_tx = self.tx.clone();
        let chat_id = self.id.fetch_add(1, SeqCst);

        loop {
            tokio::select! {
                evt = rx.next() => {
                    let frame = evt.unwrap();
                    match frame.opcode() {
                        Opcode::TEXT => {
                            chat_tx.send((chat_id, frame.data().to_vec())).unwrap();
                        }
                        _ => break,
                    }
                },
                evt = chat_rx.recv() => {
                    if let Ok((id, data)) = evt {
                        if id != chat_id {
                            let frame = WsFrame::builder().text(data);
                            tx.write(frame).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

async fn index(_req: Request<Body>) -> std::io::Result<Response<Body>> {
    let resp: Response<Body> = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .header("Connection", "keep-alive")
        .body(CHAT_HTML.into())
        .unwrap();

    Ok(resp)
}

fn main() {
    env_logger::init();

    let router = Router::builder()
        .get("/", index)
        .ws("/chat", Chat::new())
        .finalize();

    supercruise_rs::serve("0.0.0.0:8080", router);
}
