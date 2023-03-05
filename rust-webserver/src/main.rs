use actix::{Actor, StreamHandler};
use actix_web::{get, post, web, Result, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use serde::Serialize;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use futures_util::{lock::Mutex, Stream, StreamExt};

#[derive(Serialize)]
struct MyObj {
    name: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(api_json)
            .service(ws_index)
            .service(get_stream)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
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

fn make_random_item() -> MyObj {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();

    let obj = MyObj {
        name: rand_string,
    };

    obj
}
