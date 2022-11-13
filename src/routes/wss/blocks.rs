use crate::{chain::Chain, state::State};
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{
    web::{Data, Json, Path, Payload},
    HttpRequest, Responder,
};
use actix_web_actors::ws;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

const BLOCK_DATA_INTERVAL: Duration = Duration::from_secs(5);

pub async fn blocks(req: HttpRequest, stream: Payload, path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    ws::WsResponseBuilder::new(WebSocket::new(chain, chains), &req, stream).start()
}

pub struct WebSocket {
    hb: Instant,
    chain_name: String,
    chains: Data<State>,
}

impl WebSocket {
    /// Creates a new `WebSocket`.
    pub fn new(chain_name: String, chains: Data<State>) -> WebSocket {
        println!("socket cre");
        WebSocket {
            hb: Instant::now(),
            chain_name,
            chains,
        }
    }

    /// Updates `Self::hb`.
    fn beat(&mut self) {
        self.hb = Instant::now();
    }

    /// Checks if the hearbeat is timed out.
    fn hb(&self, ctx: &mut <WebSocket as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("close: {:#?}", act.hb.elapsed());
                ctx.stop();
            } else {
                ctx.ping(b"");
            };
        });
    }

    fn serve_blocks(&self, ctx: &mut <WebSocket as Actor>::Context) {
        println!("socket starting");
        ctx.run_interval(BLOCK_DATA_INTERVAL, |act, ctx| {
            if let Ok(chain) = act.chains.get(&act.chain_name) {
                if let Ok(blocks) = chain.data.blocks.lock() {
                    if let Ok(json) = serde_json::to_string(&blocks.inner) {
                        ctx.text(json)
                    }
                };
            };
        });
    }
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("socket starting");
        if let Err(error) = self.chains.get(&self.chain_name) {
            ctx.text(error);
        } else {
            self.beat();
            self.hb(ctx);
            self.serve_blocks(ctx);
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("socket handle");
        match item {
            Ok(_) => self.beat(),

            _ => ctx.stop(),
        }
    }
}
