use std::{
    fmt::Display,
    sync::Arc,
    time::{Duration, Instant},
};

use actix::prelude::*;
use actix_web::{
    web::{self, Data, Path},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws;

use crate::{chain::Chain, state::State};

const HEARTBEAT: Duration = Duration::from_secs(5);
const INTERVAL: Duration = Duration::from_secs(6);
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WebSocketError(String);
impl Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for WebSocketError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
    fn description(&self) -> &str {
        self.0.as_ref()
    }
}

impl actix_web::error::ResponseError for WebSocketError {}

pub async fn socket(
    req: HttpRequest,
    stream: web::Payload,
    chains: Data<State>,
    path: Path<String>,
) -> Result<HttpResponse, WebSocketError> {
    let chain = path.into_inner();

    match chains.get(&chain) {
        Ok(chain) => match ws::start(WebSocket::new(chain), &req, stream) {
            Ok(ws) => Ok(ws),
            Err(error) => Err(WebSocketError(error.to_string())),
        },
        Err(error) => Err(WebSocketError(error)),
    }
}

/// The Web Socket actor.
pub struct WebSocket {
    heartbeat: Instant,
    chain: Arc<Chain>,
    mode: Mode,
}

pub enum Mode {
    Blocks,
    Txs,
    Params,
    None,
}
impl From<&str> for Mode {
    fn from(s: &str) -> Self {
        match s {
            "blocks" => Mode::Blocks,
            "txs" => Mode::Txs,
            "params" => Mode::Params,
            _ => Mode::None,
        }
    }
}

impl WebSocket {
    /// Creates a new web socket.
    pub fn new(chain: Arc<Chain>) -> Self {
        Self {
            heartbeat: Instant::now(),
            chain,
            mode: Mode::None,
        }
    }

    /// The helper method that sends ping to client every 5 seconds (HEARTBEAT).
    pub fn beat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT, |act, ctx| {
            if Instant::now().duration_since(act.heartbeat) > TIMEOUT {
                println!("stop");
                ctx.stop();
            } else {
                ctx.ping(b"");
            }
        });
    }

    /// Serves data via Web Socket channel.
    pub fn serve(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(INTERVAL, |act, ctx| match act.mode {
            Mode::Blocks => {
                if let Ok(blocks) = act.chain.data_blocks() {
                    if let Ok(json) = serde_json::to_string(&blocks) {
                        ctx.text(json)
                    }
                }
            }

            Mode::Txs => {
                if let Ok(transactions) = act.chain.data_txs() {
                    if let Ok(json) = serde_json::to_string(&transactions) {
                        ctx.text(json)
                    }
                }
            }

            Mode::Params => {
                if let Ok(params) = act.chain.data_params() {
                    if let Ok(json) = serde_json::to_string(&params) {
                        ctx.text(json)
                    }
                }
            }
            _ => (),
        });
    }
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.beat(ctx);
        self.serve(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        println!("WS: {msg:?}");
        match msg {
            Ok(ws::Message::Pong(_)) => {
                self.heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                self.mode = text.to_string().as_str().into();
            }
            _ => ctx.stop(),
        }
    }
}
