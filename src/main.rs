#[macro_use]
extern crate lazy_static;

mod peer_connection;
mod signal;

use anyhow::Result;
use peer_connection::connect;

use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use clap::Parser;

/// Start up a webrtc-media server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to use
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Audio file to play
    audio: Option<String>,

    /// Video file to play
    video: Option<String>,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let port = args.port;

    // Start up the peer connection in a separate thread
    tokio::spawn(async move {
        if let Err(e) = connect(args.audio, args.video, true).await {
            eprintln!("Error: {}", e);
        }
    });

    println!("Starting server at http://127.0.0.1:{}/", port);

    HttpServer::new(|| App::new().service(hello).service(echo))
        .bind(("127.0.0.1", port))?
        .run()
        .await.map_err(anyhow::Error::from)
}
