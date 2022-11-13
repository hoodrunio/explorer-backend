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

const TRANSACTIONS_DATA_INTERVAL: Duration = Duration::from_secs(5);

pub async fn transactions(req: HttpRequest, stream: Payload, path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    ws::start(WebSocket::new(chain, chains), &req, stream)
}

pub struct WebSocket {
    hb: Instant,
    chain_name: String,
    chains: Data<State>,
}

impl WebSocket {
    /// Creates a new `WebSocket`.
    pub fn new(chain_name: String, chains: Data<State>) -> WebSocket {
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

    fn serve_transactions(&self, ctx: &mut <WebSocket as Actor>::Context) {
        ctx.run_interval(TRANSACTIONS_DATA_INTERVAL, |act, ctx| {
            if let Ok(chain) = act.chains.get(&act.chain_name) {
                if let Ok(transactions) = chain.data.transactions.lock() {
                    if let Ok(json) = serde_json::to_string(&transactions.inner) {
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
        if let Err(error) = self.chains.get(&self.chain_name) {
            ctx.text(error);
            ctx.stop();
        } else {
            self.hb(ctx);
            self.serve_transactions(ctx);
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Pong(_)) => (),

            _ => ctx.stop(),
        }
    }
}
