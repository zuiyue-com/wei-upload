use axum::{
    extract::Multipart,
    routing::post,
    Router,
};

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

async fn upload(mut multipart: Multipart) {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string().replace("..","");
        let data = field.bytes().await.unwrap();

        let path = name.clone();
        let path = std::path::Path::new(&path);
        let current_dir = std::env::current_dir().unwrap();
        if let Some(parent_path) = path.parent() {
            let parent_path = format!("{}/{}", current_dir.display(), parent_path.display());
            std::fs::create_dir_all(parent_path).unwrap();
        }

        let name = format!("{}/{}", current_dir.display(), name);
        
        let mut file = File::create(name.clone()).await.unwrap();
        file.write_all(&data).await.unwrap();
    
        println!("upload file `{}`", name);
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/upload", post(upload));

    let address = format!("0.0.0.0:8001");
    println!("Server running on {}", address);
    axum::Server::bind(&address.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
