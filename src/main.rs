#[macro_use]
extern crate lazy_static;

mod peer_connection;
mod signal;

use actix_cors::Cors;
use anyhow::Result;
use peer_connection::connect;

use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use clap::Parser;

/// Start up a webrtc-media server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to use
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Audio file to play
    #[arg(short, long)]
    audio: Option<String>,

    /// Video file to play
    #[arg(short, long)]
    video: Option<String>,
}

#[post("/api/watch")]
async fn webrtc_offer(req_body: String) -> impl Responder {
    let args = Args::parse();

    let peer_connection = connect(args.audio, args.video, &req_body, false).await;

    // transform the below into a match statement
    match peer_connection {
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let port = args.port;

    println!("Starting server at http://127.0.0.1:{port}/");

    HttpServer::new(|| App::new().wrap(Cors::permissive()).service(webrtc_offer))
        .bind(("127.0.0.1", port))?
        .disable_signals()
        .run()
        .await
        .map_err(anyhow::Error::from)
}
