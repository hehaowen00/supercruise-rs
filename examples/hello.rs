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
    let mut f = tokio::fs::File::open("client.html").await?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).await?;

    let resp: Response<Body> = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .header("Connection", "close")
        .body(buf.clone().into())
        .unwrap();

    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new().get("/", index).ws("/ws", Hello {});

    supercruise_rs::serve(router).await
}
