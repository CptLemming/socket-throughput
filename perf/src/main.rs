use std::time::{Instant, Duration};
use actix::{
  io::SinkWrite, Actor, ActorContext, AsyncContext, Context, Handler, Message,
  StreamHandler, WrapFuture, ActorFutureExt,
};
use actix::prelude::*;
use actix_web::{HttpServer, App};
use futures::TryStreamExt;
use futures_util::stream::StreamExt;
use clap::{Parser, ValueEnum};

use awc::{error::WsProtocolError, ws, Client};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[arg(value_enum)]
  mode: Mode,

  #[arg(short, long, default_value_t = 10)]
  times: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    HTTP,
    WS,
    Stream,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  let cli = Cli::parse();
  println!("{:?}", cli);

  match cli.mode {
    Mode::HTTP => {
      HttpClient{ times: cli.times, start: None, results: vec![] }.start();
    }
    Mode::WS => {
      WebSocketClient{ times: cli.times, replies: 0, start: None }.start();
    }
    Mode::Stream => {
      perform_stream_test(cli.times).await;
    }
  }

  // Start an async process to stop script ending
  HttpServer::new(|| {
      App::new()
  })
  .bind(("127.0.0.1", 0))?
  .run()
  .await
  .unwrap();

  Ok(())
}

pub struct HttpClient {
  times: u32,
  start: Option<Instant>,
  results: Vec<u128>,
}

impl Actor for HttpClient {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Context<Self>) {
    self.start = Some(Instant::now());
    ctx.address().do_send(HttpClientNext{});
  }
}

pub struct HttpClientNext;

impl Message for HttpClientNext {
    type Result = Result<(), ()>;
}

pub struct HttpClientResponse {
  duration: Duration,
}

impl Message for HttpClientResponse {
    type Result = Result<(), ()>;
}

pub struct HttpClientComplete;

impl Message for HttpClientComplete {
    type Result = Result<(), ()>;
}

impl Handler<HttpClientNext> for HttpClient {
  type Result = Result<(), ()>;

  fn handle(&mut self, _msg: HttpClientNext, ctx: &mut Self::Context) -> Self::Result {
      // println!("http next");
      let self_ref = ctx.address();

      // TODO How the hell do you handle async requests without spawning a thread?!?
      actix_web::rt::spawn(async move {
        let client = Client::new();

        let start = Instant::now();
        let mut res = client
            .get("http://localhost:3000/api/json")
            .send()
            .await
            .unwrap();

        let end = Instant::elapsed(&start);

        self_ref.do_send(HttpClientResponse{ duration: end });
      });

      Ok(())
  }
}

impl Handler<HttpClientResponse> for HttpClient {
  type Result = Result<(), ()>;

  fn handle(&mut self, msg: HttpClientResponse, ctx: &mut Self::Context) -> Self::Result {
    self.results.push(msg.duration.as_micros());

    if self.results.len() < (self.times as usize) {
      // println!("Send another");
      ctx.address().do_send(HttpClientNext{});
    } else {
      // println!("Complete!");
      ctx.address().do_send(HttpClientComplete{});
    }

    Ok(())
  }
}

impl Handler<HttpClientComplete> for HttpClient {
  type Result = Result<(), ()>;

  fn handle(&mut self, _msg: HttpClientComplete, _ctx: &mut Self::Context) -> Self::Result {

    if let Some(start) = self.start {
      let end = Instant::elapsed(&start);
      let sum: u128 = self.results.iter().sum();
      println!("Avg :: 0.{}ms", sum / (self.results.len() as u128));
      println!("Total :: {}ms", end.as_millis());
    }

    Ok(())
  }
}

pub struct WebSocketClient {
  times: u32,
  start: Option<Instant>,
  replies: u32,
}

impl Actor for WebSocketClient {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Context<Self>) {
    Client::default()
      .ws("ws://localhost:3000/ws")
        .connect()
        .into_actor(self)
        .map(|res, remote, ctx| match res {
          Ok((client_response, frame)) => {
            println!("WS started: {:?}", client_response);
              let (sink, stream) = frame.split();
              ctx.add_stream(stream);
              let mut sink_writer = SinkWrite::new(sink, ctx);
              sink_writer.write(ws::Message::Text(remote.times.to_string().into())).unwrap();
              remote.start = Some(Instant::now());
              remote.replies = 0;
          }
          Err(err) => {
            println!("Websocket Client Actor failed to connect: {:?}", err);
              ctx.stop();
          }
        }).wait(ctx);
  }
}

impl actix::io::WriteHandler<WsProtocolError> for WebSocketClient {}

impl StreamHandler<Result<ws::Frame, WsProtocolError>> for WebSocketClient {
  fn handle(&mut self, item: Result<ws::Frame, WsProtocolError>, _ctx: &mut Self::Context) {
    // println!("Response {:?} of {}", item, self.replies);
    self.replies += 1;

    if self.replies >= self.times {
      // Finished
      if let Some(start) = self.start {
        let end = Instant::elapsed(&start);
        println!("Avg :: 0.{}ms", end.as_micros() / (self.times as u128));
        println!("Avg :: {}ns", end.as_nanos() / (self.times as u128));
        println!("Total :: 0.{}ms", end.as_micros());
        println!("Total :: {}ms", end.as_millis());
        println!("Total :: {}ns", end.as_nanos());
      }
    }
  }
}

async fn perform_stream_test(times: u32) {
  let client = Client::new();
  let mut res = client
      .get(format!("http://localhost:3000/stream?times={}", times))
      .send()
      .await
      .unwrap();

  let start = Instant::now();
  let mut s = res.into_stream();

  while let Some(value) = s.next().await {
    // println!("Buffer:: {:?}", value);
  }
  let end = Instant::elapsed(&start);
  println!("Avg :: 0.{}ms", end.as_micros() / (times as u128));
  println!("Avg :: {}ns", end.as_nanos() / (times as u128));
  println!("Total :: 0.{}ms", end.as_micros());
  println!("Total :: {}ms", end.as_millis());
  println!("Total :: {}ns", end.as_nanos());
}
