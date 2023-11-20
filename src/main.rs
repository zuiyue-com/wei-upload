use axum::{
    extract::Multipart,
    routing::post,
    Router,
    Json,
};

use serde::Deserialize;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

async fn upload(mut multipart: Multipart) -> String {
    let mut return_data = serde_json::json!({
        "code": 200
    }).to_string();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string().replace("..","");
        let data = match field.bytes().await {
            Ok(data) => data,
            Err(e) => {
                return serde_json::json!({
                    "code": 400,
                    "message": format!("{:?}", e)
                }).to_string();
            }
        };

        let path = name.clone();
        let path = std::path::Path::new(&path);
        let current_dir = std::env::current_dir().unwrap();
        if let Some(parent_path) = path.parent() {
            let parent_path = format!("{}/{}", current_dir.display(), parent_path.display());
            std::fs::create_dir_all(parent_path.clone()).unwrap();
            use std::os::unix::fs::PermissionsExt;

            let perm = std::fs::Permissions::from_mode(0o777);
            std::fs::set_permissions(parent_path, perm).unwrap();
        }

        let name = format!("{}/{}", current_dir.display(), name);
        
        let mut file = File::create(name.clone()).await.unwrap();
        match file.write_all(&data).await {
            Ok(_) => {
                return_data = serde_json::json!({
                    "code": 200,
                    "path": name,
                }).to_string();
            },
            Err(_) => {
                return_data = serde_json::json!({
                    "code": 400,
                    "message": "write file failed"
                }).to_string();
            }
        }
    }

    return_data
}

use axum::{response::IntoResponse};
use hyper::{Body, Response, StatusCode};
use std::convert::Infallible;
async fn download_file(Json(req): Json<FileRequest>) -> Result<impl IntoResponse, Infallible> {


    match tokio::fs::read(req.name).await {
        Ok(data) => {
            return Ok(Response::builder()
                .header("Content-Type", "application/octet-stream")
                .header("Content-Disposition", "attachment; filename=\"your_file_name.ext\"")
                .body(Body::from(data))
                .unwrap());
        },
        Err(err) => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!("File not found: {}", err)))
                .unwrap());
        }
    }
}

#[derive(Deserialize)]
struct FileRequest {
    name: String,
}

async fn file_size(Json(req): Json<FileRequest>) -> String {
    let name = req.name.clone();

    let current_dir = std::env::current_dir().unwrap();
    let name = format!("{}/{}", current_dir.display(), name);
    let name = name.replace("..", "");

    match get_file_size(&name) {
        Ok(size) => serde_json::json!({
            "code": 200,
            "name": req.name,
            "path": name,
            "size": size
        }).to_string(),
        Err(err) => serde_json::json!({
            "code": 400,
            "name": req.name,
            "path": name,
            "message": format!("{}", err)
        }).to_string()
    }
}

async fn delete(Json(req): Json<FileRequest>) -> String {
    let name = req.name.clone();

    let current_dir = std::env::current_dir().unwrap();
    let name = format!("{}/{}", current_dir.display(), name);
    let name = name.replace("..", "");

    match std::fs::remove_file(name.clone()) {
        Ok(_) => serde_json::json!({
            "code": 200,
            "name": req.name,
            "path": name
        }).to_string(),
        Err(err) => serde_json::json!({
            "code": 400,
            "name": req.name,
            "path": name,
            "message": format!("{}", err)
        }).to_string()
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/upload", post(upload))
        .route("/download", post(download_file))
        .route("/file_size", post(file_size))
        .route("/delete", post(delete))
        .layer(axum::extract::DefaultBodyLimit::max(100*1024*1024));

    let address = format!("0.0.0.0:8001");
    println!("Server running on {}", address);
    axum::Server::bind(&address.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}


use std::fs;

fn get_file_size(file_path: &str) -> std::io::Result<u64> {
    let metadata = fs::metadata(file_path)?;
    Ok(metadata.len())
}
