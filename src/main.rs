use axum::{
    Router,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
};

use axum_extra::response::file_stream::FileStream;
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

async fn file_stream() -> Result<Response, (StatusCode, String)> {
    let file = File::open("src/videos/stutter.mp4")
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("File not found: {e}")))?;

    // Create the basic file stream
    let mut file_stream_resp = FileStream::new(ReaderStream::new(file)).file_name("lmao.mp4");

    // Convert to a response so we can modify headers
    let mut response = file_stream_resp.into_response();

    // Set the Content-Type header
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("video/mp4"),
    );

    Ok(response)
}

async fn video_html() -> Html<String> {
    let content = r#"<head>
            <link href="https://vjs.zencdn.net/8.16.1/video-js.css" rel="stylesheet" />

            <!-- If you'd like to support IE8 (for Video.js versions prior to v7) -->
            <!-- <script src="https://vjs.zencdn.net/ie8/1.1.2/videojs-ie8.min.js"></script> -->
        </head>

        <body>
            <video
                id="my-video"
                class="video-js"
                controls
                preload="auto"
                width="640"
                height="264"
                poster="MY_VIDEO_POSTER.jpg"
                data-setup="{}"
            >
                <source
                    src="http://localhost:3000/video/stutter.mp4"
                    type="video/mp4"
                />

                <p class="vjs-no-js">
                    To view this video please enable JavaScript, and consider upgrading
                    to a web browser that
                    <a href="https://videojs.com/html5-video-support/" target="_blank"
                        >supports HTML5 video</a
                    >
                </p>
            </video>

            <script src="https://vjs.zencdn.net/8.16.1/video.min.js"></script>
        </body>"#;

    Html(content.to_string())
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
        .route("/video/stutter.mp4", get(file_stream))
        .route("/", get(video_html))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("listening on port: {:?}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    println!();
}
