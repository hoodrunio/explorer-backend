use std::{
    collections::VecDeque,
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

use crate::{chain::Chain, state::State};

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

/// How often heartbeat pings are sent
const _HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn socket(req: HttpRequest, stream: web::Payload, chains: Data<State>, path: Path<String>) -> Result<HttpResponse, Error> {
    let name = path.into_inner();
    match chains.get(&name) {
        Ok(chain) => ws::start(MyWebSocket::new(chain), &req, stream),
        Err(error) => Err(CustomError(error).into()),
    }
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
pub struct MyWebSocket {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    last_pong_time: Instant,

    chain: Chain,

    last_block_sent: u64,
}

impl MyWebSocket {
    pub fn new(chain: Chain) -> Self {
        Self {
            last_pong_time: Instant::now(),
            chain,
            last_block_sent: 0,
        }
    }

    pub fn update_last_pong_time(&mut self) {
        self.last_pong_time = Instant::now();
    }

    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.spawn(
            IntervalFunc::new(Duration::from_secs_f64(
                *self.chain.inner.data.last_ten_blocks.avg_block_time_secs.lock().unwrap(),
            ))
            .finish(),
        );
    }
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        println!("WS: {msg:?}");
        match msg {
            Ok(ws::Message::Pong(_)) => {
                self.update_last_pong_time();
            }
            _ => ctx.stop(),
        }
    }
}

pin_project! {
    /// An `ActorStream` that periodically runs a function in the actor's context.
    ///
    /// Unless you specifically need access to the future, use [`Context::run_interval`] instead.
    ///
    /// [`Context::run_interval`]: ../prelude/trait.AsyncContext.html#method.run_interval
    ///
    /// ```
    /// # use std::io;
    /// use std::time::Duration;
    /// use actix::prelude::*;
    /// use actix::utils::IntervalFunc;
    ///
    /// struct MyActor;
    ///
    /// impl MyActor {
    ///     fn tick(&mut self, context: &mut Context<Self>) {
    ///         println!("tick");
    ///     }
    /// }
    ///
    /// impl Actor for MyActor {
    ///    type Context = Context<Self>;
    ///
    ///    fn started(&mut self, context: &mut Context<Self>) {
    ///        // spawn an interval stream into our context
    ///        IntervalFunc::new(Duration::from_millis(100), Self::tick)
    ///            .finish()
    ///            .spawn(context);
    /// #      context.run_later(Duration::from_millis(200), |_, _| System::current().stop());
    ///    }
    /// }
    /// # fn main() {
    /// #    let mut sys = System::new();
    /// #    let addr = sys.block_on(async { MyActor.start() });
    /// #    sys.run();
    /// # }
    /// ```
    #[must_use = "future do nothing unless polled"]
    pub struct IntervalFunc {
        #[pin]
        timer: tokio::time::Sleep,
    }
}

impl IntervalFunc {
    /// Creates a new `IntervalFunc` with the given interval duration.
    pub fn new(dur: Duration) -> IntervalFunc {
        Self {
            timer: tokio::time::sleep(dur),
        }
    }
}

impl ActorStream<MyWebSocket> for IntervalFunc {
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

            // Stop the connection if the client timeout is passed.
            if Instant::now().duration_since(act.last_pong_time) > CLIENT_TIMEOUT {
                ctx.stop();
            } else {
                if let Ok(last_ten_blocks_queue) = act.chain.inner.data.last_ten_blocks.queue.lock() {
                    let last_ten_blocks = last_ten_blocks_queue.clone();
                    drop(last_ten_blocks_queue);

                    let blocks_count = last_ten_blocks.len();

                    let mut que = VecDeque::new();

                    for i in 1..blocks_count {
                        let index = blocks_count - i;
                        if let Some(block) = last_ten_blocks.get(index) {
                            que.push_back(block);
                            if block.height == act.last_block_sent + 1 {
                                break;
                            }
                        }
                    }

                    for block in que {
                        if let Ok(json) = serde_json::to_string(block) {
                            ctx.text(json)
                        }
                    }
                }
            }
        }
    }
}
