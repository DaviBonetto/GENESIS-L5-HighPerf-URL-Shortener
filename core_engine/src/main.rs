//! GENESIS Engine - High-Performance URL Shortener
//! 
//! Part of the Titan Protocol Initiative.
//! Engineered for O(1) lookups using probabilistic data structures.

mod storage;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use storage::BloomStore;

/// Application state shared across all requests
struct AppState {
    bloom: Arc<BloomStore>,
}

#[derive(Deserialize)]
struct CreateLinkRequest {
    url: String,
}

#[derive(Serialize)]
struct CreateLinkResponse {
    short_code: String,
    original_url: String,
    status: String,
}

#[derive(Serialize)]
struct ResolveResponse {
    short_code: String,
    message: String,
    bloom_status: String,
}

/// Health check endpoint
#[get("/health")]
async fn health_check(data: web::Data<AppState>) -> impl Responder {
    let memory_kb = data.bloom.memory_usage() / 1024;
    HttpResponse::Ok().body(format!(
        "Genesis Engine: OPERATIONAL 🟢\nBloom Filter Memory: {} KB",
        memory_kb
    ))
}

/// Create a shortened URL
/// 
/// Generates a unique short code, stores it in the Bloom Filter,
/// and returns the mapping.
#[post("/shorten")]
async fn shorten_url(
    data: web::Data<AppState>,
    body: web::Json<CreateLinkRequest>,
) -> impl Responder {
    // Generate unique short code (first 8 chars of UUID)
    let short_code = Uuid::new_v4()
        .to_string()
        .chars()
        .take(8)
        .collect::<String>();

    // Store in Bloom Filter for fast lookups
    data.bloom.add(&short_code);
    
    println!("✨ CREATED: {} -> {}", short_code, body.url);

    HttpResponse::Created().json(CreateLinkResponse {
        short_code,
        original_url: body.url.clone(),
        status: "created".to_string(),
    })
}

/// Resolve a short code to its original URL
/// 
/// Uses Bloom Filter as L2 cache to block requests for non-existent codes.
#[get("/{short_code}")]
async fn resolve_url(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let short_code = path.into_inner();
    
    // L2 Bloom Filter Check - O(1) operation
    if !data.bloom.contains(&short_code) {
        println!("⛔ BLOCKED by Bloom: {} (definitely not in store)", short_code);
        
        return HttpResponse::NotFound().json(ResolveResponse {
            short_code,
            message: "Short code not found".to_string(),
            bloom_status: "blocked".to_string(),
        });
    }

    // Bloom says it MIGHT exist (could be false positive)
    println!("✅ PASSED Bloom: {} (might exist, check DB)", short_code);
    
    // TODO: L3 - Query Redis/Postgres for actual URL
    // For now, return success indicating Bloom passed
    HttpResponse::Ok().json(ResolveResponse {
        short_code,
        message: "Bloom check passed - would query database".to_string(),
        bloom_status: "passed".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("══════════════════════════════════════════════════════════════════════");
    println!("🚀 GENESIS Engine starting on port 8080...");
    println!("🔥 High-Performance Mode: ACTIVATED");
    println!("🧬 L2 Bloom Filter: ONLINE (1M capacity, 1% FPR)");
    println!("══════════════════════════════════════════════════════════════════════");

    // Initialize shared state with Bloom Filter
    let bloom_store = Arc::new(BloomStore::new());
    
    HttpServer::new(move || {
        let app_state = AppState {
            bloom: Arc::clone(&bloom_store),
        };
        
        App::new()
            .app_data(web::Data::new(app_state))
            .service(health_check)
            .service(shorten_url)
            .service(resolve_url)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
