use std::{
    fmt::Display,
    pin::Pin,
    task::Poll,
    time::{Duration, Instant},
};

use actix::prelude::*;
use actix_web::{
    web::{self, Data, Path},
    Error, HttpRequest, HttpResponse, ResponseError,
};
use actix_web_actors::ws;
use futures_core::ready;
use futures_core::Future;
use pin_project_lite::pin_project;
use serde::Serialize;

use crate::{
    chain::Chain,
    fetch::{blocks::BlockItem, transactions::TransactionItem},
    state::State,
};

//
//
//
//  Send "subscribe_tx_"
//
//
//
//
//
//
//
//
//
//
//
//
//

#[derive(Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SocketResponse {
    Block(BlockItem),
    Tx(TransactionItem),
}

/// How long before lack of client response causes a timeout.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// How long before another ping message is sent.
const TIMEOUT_CHECK_DURATION: Duration = Duration::from_secs(5);

/// How long before another batch of new Txs sent.
const NEW_TX_DURATION: Duration = Duration::from_secs(4);

/// Web Socket Actor for all the Web Socket connections.
pub struct MyWebSocket {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    last_pong_time: Instant,

    /// The chain inside. It is just a pointer, cuz `Chain` uses `Arc` inside.
    chain: Chain,

    last_block_num_sent: u64,
    last_tx_hash_sent: String,

    is_subscribed_to_blocks: bool,
    is_subscribed_to_txs: bool,
}

impl MyWebSocket {
    pub fn new(chain: Chain) -> Self {
        Self {
            last_pong_time: Instant::now(),
            chain,

            last_block_num_sent: 0,
            last_tx_hash_sent: String::from(""),

            is_subscribed_to_blocks: false,
            is_subscribed_to_txs: false,
        }
    }
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.spawn(CheckTimeOutFunc::new().finish());
        ctx.spawn(NewBlocksFunc::new(&self.chain).finish());
        ctx.spawn(NewTxsFunc::new().finish());
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        match msg {
            Ok(msg) => match msg {
                ws::Message::Text(text) => match text.to_string().as_ref() {
                    "block+" => self.is_subscribed_to_blocks = true,
                    "block-" => self.is_subscribed_to_blocks = false,
                    "tx+" => self.is_subscribed_to_txs = true,
                    "tx-" => self.is_subscribed_to_txs = false,
                    _ => (),
                },

                ws::Message::Pong(_) => {
                    self.last_pong_time = Instant::now();
                }

                _ => (),
            },

            Err(_) => ctx.stop(),
        }
    }
}

pin_project! {
    #[must_use = "future do nothing unless polled"]
    pub struct CheckTimeOutFunc {
        #[pin]
        timer: tokio::time::Sleep,
    }
}

pin_project! {
    #[must_use = "future do nothing unless polled"]
    pub struct NewBlocksFunc {
        #[pin]
        timer: tokio::time::Sleep,
    }
}

pin_project! {
    #[must_use = "future do nothing unless polled"]
    pub struct NewTxsFunc {
        #[pin]
        timer: tokio::time::Sleep,
    }
}

impl CheckTimeOutFunc {
    /// Creates a new `CheckTimeOutFunc` with the given interval duration.
    pub fn new() -> Self {
        Self {
            timer: tokio::time::sleep(TIMEOUT_CHECK_DURATION),
        }
    }
}

impl NewBlocksFunc {
    /// Creates a new `NewBlocksFunc` with the given interval duration.
    pub fn new(chain: &Chain) -> Self {
        Self {
            timer: tokio::time::sleep(Duration::from_secs_f64(
                *chain.inner.data.last_ten_blocks.avg_block_time_secs.lock().unwrap(),
            )),
        }
    }
}

impl NewTxsFunc {
    /// Creates a new `NewTxsFunc` with the given interval duration.
    pub fn new() -> Self {
        Self {
            timer: tokio::time::sleep(NEW_TX_DURATION),
        }
    }
}

impl ActorStream<MyWebSocket> for CheckTimeOutFunc {
    type Item = ();

    fn poll_next(
        self: Pin<&mut Self>,
        act: &mut MyWebSocket,
        ctx: &mut <MyWebSocket as Actor>::Context,
        task: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            ready!(this.timer.as_mut().poll(task));
            let now = this.timer.deadline();
            this.timer.as_mut().reset(now + TIMEOUT_CHECK_DURATION);

            // Stop the connection if the client timeout is passed.
            if Instant::now().duration_since(act.last_pong_time) > CLIENT_TIMEOUT {
                ctx.stop();
            } else {
                ctx.ping(b"");
            }
        }
    }
}

impl ActorStream<MyWebSocket> for NewBlocksFunc {
    type Item = ();

    fn poll_next(
        self: Pin<&mut Self>,
        act: &mut MyWebSocket,
        ctx: &mut <MyWebSocket as Actor>::Context,
        task: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            ready!(this.timer.as_mut().poll(task));
            let now = this.timer.deadline();
            this.timer
                .as_mut()
                .reset(now + Duration::from_secs_f64(*act.chain.inner.data.last_ten_blocks.avg_block_time_secs.lock().unwrap()));

            if act.is_subscribed_to_blocks {
                if let Some(blocks) = act.chain.inner.data.last_ten_blocks.get_blocks_till(act.last_block_num_sent) {
                    for block in blocks {
                        let block_height = block.height;

                        if let Ok(block_json_string) = serde_json::to_string(&SocketResponse::Block(block)) {
                            ctx.text(block_json_string);
                            act.last_block_num_sent = block_height;
                        }
                    }
                }
            }
        }
    }
}

impl ActorStream<MyWebSocket> for NewTxsFunc {
    type Item = ();

    fn poll_next(
        self: Pin<&mut Self>,
        act: &mut MyWebSocket,
        ctx: &mut <MyWebSocket as Actor>::Context,
        task: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            ready!(this.timer.as_mut().poll(task));
            let now = this.timer.deadline();
            this.timer.as_mut().reset(now + NEW_TX_DURATION);

            if act.is_subscribed_to_txs {
                if let Some(txs) = act.chain.inner.data.last_ten_txs.get_txs_till(&act.last_tx_hash_sent) {
                    for tx in txs {
                        let tx_hash = tx.hash.clone();

                        if let Ok(tx_json_string) = serde_json::to_string(&SocketResponse::Tx(tx)) {
                            ctx.text(tx_json_string);
                            act.last_tx_hash_sent = tx_hash;
                        }
                    }
                }
            }
        }
    }
}

/// It is the route for all the Web Socket connections.
pub async fn socket(req: HttpRequest, stream: web::Payload, chains: Data<State>, path: Path<String>) -> Result<HttpResponse, Error> {
    let name = path.into_inner();
    match chains.get(&name) {
        Ok(chain) => ws::start(MyWebSocket::new(chain), &req, stream),
        Err(error) => Err(CustomError(error).into()),
    }
}

/// It is the error type for Web Socket connections.
///
/// It sends a `404 Not Found` response displaying the error message given.
#[derive(Debug)]
pub struct CustomError(String);

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for CustomError {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::NOT_FOUND
    }
}
