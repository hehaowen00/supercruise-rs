use async_trait::async_trait;
use supercruise_rs::prelude::*;
use supercruise_rs::route::Route;

use tokio::io::AsyncReadExt;

struct Hello;

#[async_trait]
impl Route<Ws> for Hello {
    async fn handle(&self, ctx: &mut Context<Ws>) -> std::io::Result<()> {
        let msg = ctx.next().await?;

        println!("received {:?}", msg);

        if msg.opcode() == &Opcode::CLOSE || !msg.masked() {
            let resp = WsFrame::builder().close();

            ctx.send(resp).await?;

            return Ok(());
        }

        let mut data = Vec::with_capacity(255);
        for _ in 0..6 {
            for c in 'a'..='z' {
                data.push(c as u8);
            }
        }

        let resp = WsFrame::builder().text(data);

        ctx.send(resp).await?;

        Ok(())
    }
}

async fn index(_req: Request<Body>) -> std::io::Result<Response<Body>> {
    // let mut f = tokio::fs::File::open("client.html").await?;
    // let mut buf = Vec::new();
    // f.read_to_end(&mut buf).await?;

    let resp: Response<Body> = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .header("Connection", "keep-alive")
        .body("Hello, World!".into())
        .unwrap();

    Ok(resp)
}

fn main() {
    env_logger::init();
    let router = Router::builder()
        .get("/", index)
        .ws("/ws", Hello {})
        .finalize();

    supercruise_rs::start_server("0.0.0.0:8080", router);
}
