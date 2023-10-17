use axum::{
    extract::Multipart,
    routing::post,
    Router,
};
// use futures_util::stream::StreamExt;

async fn upload(mut multipart: Multipart) {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Length of `{}` is {} bytes", name, data.len());
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/upload", post(upload));

    let address = format!("127.0.0.1:8000");
    println!("Server running on {}", address);
    axum::Server::bind(&address.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
