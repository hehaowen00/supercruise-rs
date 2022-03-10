use supercruise_rs::codec::websocket::*;
use async_trait::async_trait;
use supercruise_rs::Router;
use supercruise_rs::flow::Context;
use supercruise_rs::route::Half;
use supercruise_rs::common::File;

struct Hello;

#[async_trait]
impl Half<Ws> for Hello {
    async fn handle(&self, ctx: &mut Context<Ws>) -> std::io::Result<()> {
        let msg = ctx.next().await?;

        println!("received {:?}", msg);

        if msg.opcode() == &Opcode::CLOSE || !msg.masked() {
            let resp = WsFrame::builder()
                .close();

            ctx.send(resp).await?;

            return Ok(());
        }

        let mut xs = vec![];

        let resp = WsFrame::builder()
            .close();

        xs.push(resp);

        let resp = WsFrame::builder()
            .text(String::from("goodbye world"));

        xs.push(resp);

        for x in xs.pop() {
            ctx.send(x).await?;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/", File::new("client.html"))
        .ws("/ws", Hello{});

    supercruise_rs::serve(router).await
}

