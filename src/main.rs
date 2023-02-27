#[macro_use] extern crate log;

mod peer_connection;
mod media_type;

use actix_cors::Cors;
use anyhow::Result;
use peer_connection::{connect, handle};

use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use clap::Parser;

use crate::media_type::MediaType;

/// Start up a webrtc-media server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to use
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Host to use (default is 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Media sources to play from
    media: Vec<String>,

    /// Whether to loop the audio/video files
    #[arg(short, long)]
    loop_: bool,
}

#[post("/api/watch")]
async fn webrtc_offer(req_body: String) -> impl Responder {
    let args = Args::parse();

    let media = args.media.get(0).unwrap();

    let media_type = if media.starts_with("file:") {
        MediaType::File(media.replace("file:", ""))
    } else if media.starts_with("v4l2:") {
        MediaType::Video4Linux2(media.replace("v4l2:", ""))
    } else {
        MediaType::File(media.to_string())
    };

    let peer_connection = connect(media_type, &req_body).await;

    match peer_connection {
        Ok(connection) => {
            let description = serde_json::to_string(&connection.description);
            
            tokio::spawn(handle(connection));

            match description {
                Ok(answer) => HttpResponse::Ok().body(answer),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string())
            }
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = Args::parse();
    let port = args.port;
    let host = args.host;

    info!("Starting server at http://{host}:{port}/");

    HttpServer::new(|| App::new().wrap(Cors::default().allow_any_origin().send_wildcard()).service(webrtc_offer))
        .bind((host, port))?
        .run()
        .await
        .map_err(anyhow::Error::from)
}
