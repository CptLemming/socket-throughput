use std::fmt::Display;
use std::io::Write;

use actix_web::rt::net::{TcpListener, TcpStream};
use actix_web::rt::signal;
use actix_web::web::BufMut;
use deku::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::{get, post, web, Result, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use serde::Serialize;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use futures_util::{lock::Mutex, Stream, StreamExt};
use tokio::io::AsyncWriteExt;

#[derive(Serialize)]
struct MyObj {
    name: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let socket = make_socket_server();
    let http = HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(api_json)
            .service(ws_index)
            .service(get_stream)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 3000))?
    .run();

    tokio::join!(socket, http, signal::ctrl_c());

    Ok(())
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/api/json")]
async fn api_json() -> Result<impl Responder> {
    Ok(web::Json(make_random_item()))
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

struct MyWs;

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                println!("recv {}", text);

                let num_messages: Result<u128, _> = text.parse();

                match num_messages {
                  Ok(num) => {
                    for x in 0..num {
                      // ctx.text(make_random_item());
                      ctx.text(serde_json::to_string(&serde_json::json!(make_random_item())).unwrap());
                    }
                  }
                  Err(error) => {
                    println!("Not a valid number")
                  }
                }
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

#[get("/ws")]
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWs {}, &req, stream)
}

#[derive(Debug, serde::Deserialize)]
pub struct StreamRequestParams {
   times: u64,
}

#[get("/stream")]
async fn get_stream(info: web::Query<StreamRequestParams>) -> Result<HttpResponse, Error> {
    let mut value = 0;
    let n = 1;

    let stream = async_stream::stream! {
        loop {
            if value >= info.times {
                break;
            }
            if value != 0 {
                yield make_stream_bookend(",");
            }
            yield make_stream_item();
            value += n;
        }
    };

    let first = async_stream::stream! {
        yield make_stream_bookend("[")
    };
    let last = async_stream::stream! {
        yield make_stream_bookend("]")
    };

    Ok(HttpResponse::Ok().streaming(Box::pin(first.chain(stream).chain(last))))
}

fn make_stream_bookend(char: &str) -> Result<web::Bytes, Error> {
    Ok(web::Bytes::from(char.to_string()))
}

fn make_stream_item() -> Result<web::Bytes, Error> {
    Ok(web::Bytes::from(serde_json::to_string(&serde_json::json!(make_random_item())).unwrap()))
}

fn make_uuid() -> String {
  thread_rng()
        .sample_iter(&Alphanumeric)
        .take(36)
        .map(char::from)
        .collect()
}

fn make_random_item() -> MyObj {
    let rand_string: String = make_uuid();

    let obj = MyObj {
        name: rand_string,
    };

    obj
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

async fn make_socket_server() {
  let handler = tokio::spawn(async move {
    let listener = TcpListener::bind("127.0.0.1:3001").await.unwrap();

    loop {
      let (socket, _) = listener.accept().await.unwrap();
      process_socket(socket).await;
    }
  });

  tokio::select! {
    _ = handler => {
        println!("Socket stopped");
    }
    _ = signal::ctrl_c() => {
        println!("CTRL+C called");
    }
  };
}

async fn process_socket(mut socket: TcpStream) {
  println!("Socket connected");
  loop {
    socket.readable().await.unwrap();

    let mut buf = [0; 256];

    match socket.try_read(&mut buf) {
      Ok(0) => {
        println!("Socket closed");
        break;
      }
      Ok(size) => {
        println!("reading from stream {}", size);
        // println!("buffer -> {:?}", buf);

        let (_remaining, req) = SocketRequest::from_bytes((buf.as_ref(), 0)).unwrap();
        println!("Send response {} times", req.times);
        let mut out_buf = vec![];
        let mut out_buf_writer = out_buf.writer();

        for _ in 0..req.times {
          let uuid = make_uuid();
          let res = SocketResponse{ name: AdvString { str_len: uuid.len() as i32, str_bytes: uuid.into_bytes() } };
          out_buf_writer.write(&res.to_bytes().unwrap()).unwrap();
        }

        socket.write(&out_buf_writer.into_inner()).await.unwrap();

        // for _ in 0..req.times {
        //   let uuid = make_uuid();
        //   let res = SocketResponse{ name: AdvString { str_len: uuid.len() as i32, str_bytes: uuid.into_bytes() } };
        //   socket.write(&res.to_bytes().unwrap()).await.unwrap();
        // }
      }
      Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
        continue;
      }
      Err(err) => {
        println!("{:?}", err);
      }
    }
  }
}
