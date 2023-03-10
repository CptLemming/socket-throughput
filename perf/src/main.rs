use std::fmt::Display;
use std::time::{Instant, Duration};
use actix::{
  io::SinkWrite, Actor, ActorContext, AsyncContext, Context, Handler, Message,
  StreamHandler, WrapFuture, ActorFutureExt,
};
use deku::prelude::*;
use actix::prelude::*;
use actix_rt::net::{TcpSocket, TcpStream};
use actix_web::{HttpServer, App};
use futures::TryStreamExt;
use futures_util::stream::StreamExt;
use clap::{Parser, ValueEnum};

use awc::{error::WsProtocolError, ws, Client};
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;
use env_logger::Builder;
use log::LevelFilter;

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
    Socket,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  Builder::new()
        .format_timestamp_millis()
        .filter(None, LevelFilter::Info)
        .init();

  let cli = Cli::parse();
  log::info!("{:?}", cli);

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
    Mode::Socket => {
      perform_socket_test(cli.times).await;
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
      log::info!("Avg :: 0.{}ms", sum / (self.results.len() as u128));
      log::info!("Total :: {}ms", end.as_millis());
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
            log::info!("WS started: {:?}", client_response);
              let (sink, stream) = frame.split();
              ctx.add_stream(stream);
              let mut sink_writer = SinkWrite::new(sink, ctx);
              sink_writer.write(ws::Message::Text(remote.times.to_string().into())).unwrap();
              remote.start = Some(Instant::now());
              remote.replies = 0;
          }
          Err(err) => {
            log::error!("Websocket Client Actor failed to connect: {:?}", err);
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
        log::info!("Avg :: 0.{}ms", end.as_micros() / (self.times as u128));
        log::info!("Avg :: {}ns", end.as_nanos() / (self.times as u128));
        log::info!("Total :: 0.{}ms", end.as_micros());
        log::info!("Total :: {}ms", end.as_millis());
        log::info!("Total :: {}ns", end.as_nanos());
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
  log::info!("Avg :: 0.{}ms", end.as_micros() / (times as u128));
  log::info!("Avg :: {}ns", end.as_nanos() / (times as u128));
  log::info!("Total :: 0.{}ms", end.as_micros());
  log::info!("Total :: {}ms", end.as_millis());
  log::info!("Total :: {}ns", end.as_nanos());
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
// #[deku(endian = "little")]
pub struct SocketRequest {
  pub times: u32,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
// #[deku(endian = "little")]
pub struct SocketResponse {
  pub name: AdvString,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct AdvString {
  str_len: i32,
  #[deku(count = "str_len")]
  str_bytes: Vec<u8>,
}

impl AdvString {
  pub fn to_string(&self) -> String {
    return String::from_utf8_lossy(&self.str_bytes).into();
  }
}

impl From<AdvString> for String {
    fn from(src: AdvString) -> Self {
      return src.to_string();
    }
}

impl Display for AdvString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "{}", self.to_string())
    }
}

async fn perform_socket_test(times: u32) {
  log::info!("Start socket client");
  let (complete_tx, complete_rx) = oneshot::channel::<bool>();

  tokio::spawn(async move {
    let mut replies = 0;
    let mut socket = TcpStream::connect("localhost:3001").await.unwrap();

    let req = SocketRequest{ times };
    let data: Vec<u8> = req.to_bytes().unwrap();
    let mut end: Option<Duration> = None;
    socket.write(&data).await.unwrap();
    let start = Instant::now();

    loop {
      socket.readable().await.unwrap();

      let mut buf = [0; 48_000];

      match socket.try_read(&mut buf) {
        Ok(0) => {
          log::info!("Socket closed");
          break;
        }
        Ok(size) => {
          // log::info!("reading from stream {}", size);

          let mut buffer_len = size;
          loop {
            // log::info!("reading");
            let message_size = 24 + 4;
            // let (remaining, res) = SocketResponse::from_bytes((buf.as_ref(), 0)).unwrap();
            replies += 1;

            buffer_len -= message_size;

            if buffer_len <= 0 {
              // reached the end of the written buffer, wait for next message
              break;
            }
          }

          if replies >= times {
            // log::info!("complete");
            end = Some(Instant::elapsed(&start));
            break;
          }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
          continue;
        }
        Err(err) => {
          log::error!("Socket error -> {:?}", err);
        }
      }
    }

    if let Some(end) = end {
      log::info!("Avg :: 0.{}ms", end.as_micros() / (times as u128));
      log::info!("Avg :: {}ns", end.as_nanos() / (times as u128));
      log::info!("Total :: 0.{}ms", end.as_micros());
      log::info!("Total :: {}ms", end.as_millis());
      log::info!("Total :: {}ns", end.as_nanos());
    }
    complete_tx.send(true).unwrap();
  });

  complete_rx.await.unwrap();
  log::info!("Finish socket client");
}
