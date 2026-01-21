use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateLink {
    url: String,
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Genesis Engine: OPERATIONAL 🟢")
}

#[post("/shorten")]
async fn shorten_url(link: web::Json<CreateLink>) -> impl Responder {
    // Placeholder para Bloom Filter e Lógica de Hashing
    let mock_short = "xyz123";
    HttpResponse::Ok().body(format!("Shortened '{}' to '{}'", link.url, mock_short))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 GENESIS Engine starting on port 8080...");
    println!("🔥 High-Performance Mode: ACTIVATED");

    HttpServer::new(|| {
        App::new()
            .service(health_check)
            .service(shorten_url)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
