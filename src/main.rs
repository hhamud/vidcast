use std::io::Cursor;
use std::{collections::HashMap, sync::Arc};

use axum::{
    Router,
    extract::{DefaultBodyLimit, Multipart, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};

use axum_extra::response::file_stream::FileStream;
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, net::TcpListener, sync::Mutex};
use tokio_util::io::ReaderStream;

#[derive(Debug, Parser)]
struct Command {
    #[arg(short, long)]
    video_path: Option<String>,
}

async fn file_stream(State(state): State<Database>, name: String) -> Result<Response, StatusCode> {
    let file = state.get(name).await?;

    // Create the basic file stream
    let file_stream_resp =
        FileStream::new(ReaderStream::new(Cursor::new(file))).file_name("lmao.mp4");

    // Convert to a response so we can modify headers
    let mut response = file_stream_resp.into_response();

    // Set the Content-Type header
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("video/mp4"),
    );

    Ok(response)
}

async fn frontend() -> Html<String> {
    let content = r#"<head>
            <link href="https://vjs.zencdn.net/8.16.1/video-js.css" rel="stylesheet" />
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

            <div class="upload-form">
                <h2>Upload Video</h2>
                <form action="/upload" method="post" enctype="multipart/form-data">
                    <label for="file">Select a video file:</label>
                    <input type="file" id="file" name="file" accept="video/*"><br>
                    <input type="submit" value="Upload">
                </form>
            </div>

            <div class="section">
                <h2>Get Video by Name</h2>
                <form action="/video" method="get">
                    <div class="form-row">
                        <label for="video-name">Video name:</label>
                        <input type="text" id="video-name" name="name" required>
                    </div>
                    <input type="submit" value="Get Video" class="btn-blue">
                </form>
            </div>


            <script src="https://vjs.zencdn.net/8.16.1/video.min.js"></script>
        </body>"#;

    Html(content.to_string())
}

async fn upload(
    State(mut state): State<Database>,
    mut multipart: Multipart,
) -> Result<Response<String>, StatusCode> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        let name = field.name().unwrap().to_string();
        let data = field
            .bytes()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        println!("Length of `{}` is {} bytes", &name, &data.len());

        state.add(name, data.into()).await?;
    }

    let response = Response::builder()
        .status(StatusCode::CREATED)
        .body("OK".to_string());

    response.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Debug, Clone)]
struct Database {
    data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add(&mut self, name: String, content: Vec<u8>) -> Result<Vec<u8>, StatusCode> {
        self.data
            .lock()
            .await
            .insert(name, content)
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn get(&self, name: String) -> Result<Vec<u8>, StatusCode> {
        self.data
            .lock()
            .await
            .get(&name)
            .cloned()
            .ok_or(StatusCode::NOT_FOUND)
    }
}

#[tokio::main]
async fn main() {
    let cli = Command::parse();

    let database = Database::new();

    let app = Router::new()
        .route("/", get(frontend))
        .route("/video", get(file_stream))
        .route("/upload", post(upload))
        .layer(DefaultBodyLimit::max(4096))
        .with_state(database);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("listening on port: {:?}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
