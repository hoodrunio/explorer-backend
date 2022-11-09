use std::time::{Duration, Instant};

use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web_actors::ws;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebSocket {
    hb: Instant,
}

impl WebSocket {
    /// Creates a new `WebSocket`.
    pub fn new() -> WebSocket {
        WebSocket { hb: Instant::now() }
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
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx)
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // Match `item`.
        match item {
            Ok(ws::Message::Text(_)) => {
                self.beat();

                ctx.text("");
            }

            Ok(ws::Message::Pong(_)) => (),

            _ => ctx.stop(),
        }
    }
}
