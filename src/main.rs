use axum::{
    Router,
    http::{StatusCode,header},
    response::Json,
    response::{IntoResponse, Response},
    routing::{get, post},
};

use axum_extra::response::file_stream::{FileStream};
use clap::Parser;
use serde_json::{Value, json};
use tokio::fs::File;
use tokio::net::TcpListener;
use tokio_util::io::ReaderStream;

#[derive(Debug, Parser)]
struct Command {
    #[arg(short, long)]
    video_path: Option<String>,
}
use tower_http::cors::{Any, CorsLayer};

// frame frontend
// 1. video player that fetches content from the server
// 2.

// vidcast:
// 1. user uploads a video
// 2. video is converted into suitable format
// 3. algorithm compresses
// 4. a new account is purchased
// 5. that account is used to store the bytes of data to the hub
//  5a. unique id per user --> linked to a buch of casts

// `Json` gives a content-type of `application/json` and works with any type
// that implements `serde::Serialize`
async fn video(path: String) -> Json<Value> {
    let v = format!("welcome to the video link: {:?}", path);
    Json(json!({ "video": v }))
}

async fn file_stream() -> Result<Response, (StatusCode, String)> {
    let file = File::open("src/videos/stutter.mp4").await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            format!("File not found: {e}")
        )
    })?;

    // Create the basic file stream
    let mut file_stream_resp = FileStream::new(ReaderStream::new(file))
        .file_name("lmao.mp4");

    // Convert to a response so we can modify headers
    let mut response = file_stream_resp.into_response();

    // Set the Content-Type header
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("video/mp4")
    );

    Ok(response)
}


#[tokio::main]
async fn main() {
    let cli = Command::parse();

    // In your main function
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // build our application with a single route
    let app = Router::new()
        .route("/video/lmao.mp4", get(file_stream))
        .route("/video", post(video))
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!();
}
