//! GENESIS Engine - High-Performance URL Shortener
//!
//! Part of the Titan Protocol Initiative.
//! Engineered for O(1) lookups using probabilistic data structures.
//!
//! Architecture:
//! - L2: Bloom Filter (probabilistic existence check)
//! - L3: In-Memory HashMap (actual URL storage)
//! - Future: Redis (L3) + Postgres (L4)

mod storage;
mod utils;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use storage::BloomStore;
use utils::base62;

/// Application state shared across all requests
struct AppState {
    bloom: Arc<BloomStore>,
    db: Mutex<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct CreateLinkRequest {
    url: String,
}

#[derive(Serialize)]
struct CreateLinkResponse {
    short_url: String,
    original_url: String,
    short_code: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: u16,
}

/// Health check endpoint
#[get("/health")]
async fn health_check(data: web::Data<AppState>) -> impl Responder {
    let memory_kb = data.bloom.memory_usage() / 1024;
    let db_size = data.db.lock().len();
    
    HttpResponse::Ok().body(format!(
        "Genesis Engine: OPERATIONAL 🟢\n\
         Bloom Filter Memory: {} KB\n\
         URLs in Database: {}",
        memory_kb, db_size
    ))
}

/// Create a shortened URL
///
/// 1. Generate random u64
/// 2. Encode to Base62 short code
/// 3. Store in Bloom Filter + HashMap
/// 4. Return short URL
#[post("/shorten")]
async fn shorten_url(
    data: web::Data<AppState>,
    body: web::Json<CreateLinkRequest>,
) -> impl Responder {
    // Generate random ID and encode to Base62
    let random_id: u64 = rand::random();
    let short_code = base62::encode(random_id);
    
    // Store in Bloom Filter (L2)
    data.bloom.add(&short_code);
    
    // Store in HashMap (L3)
    {
        let mut db = data.db.lock();
        db.insert(short_code.clone(), body.url.clone());
    }
    
    let short_url = format!("http://127.0.0.1:8080/{}", short_code);
    
    println!("✨ CREATED: {} -> {}", short_code, body.url);

    HttpResponse::Created().json(CreateLinkResponse {
        short_url,
        original_url: body.url.clone(),
        short_code,
    })
}

/// Resolve a short code to its original URL
///
/// Flow:
/// 1. Check Bloom Filter (L2) - O(1)
///    - If NO: Return 404 immediately (guaranteed not in DB)
/// 2. Check HashMap (L3)
///    - If found: HTTP 302 Redirect
///    - If not found: False positive (rare), return 404
#[get("/{short_code}")]
async fn resolve_url(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let short_code = path.into_inner();
    
    // Skip health endpoint
    if short_code == "health" {
        return HttpResponse::NotFound().json(ErrorResponse {
            error: "Use GET /health for health check".to_string(),
            code: 404,
        });
    }
    
    // L2: Bloom Filter Check - O(1)
    if !data.bloom.contains(&short_code) {
        println!("⛔ BLOCKED by Bloom: {}", short_code);
        
        return HttpResponse::NotFound().json(ErrorResponse {
            error: "Short code not found".to_string(),
            code: 404,
        });
    }

    // L3: HashMap Lookup
    let original_url = {
        let db = data.db.lock();
        db.get(&short_code).cloned()
    };
    
    match original_url {
        Some(url) => {
            println!("✅ REDIRECT: {} -> {}", short_code, url);
            
            // HTTP 302 Found - Redirect to original URL
            HttpResponse::Found()
                .append_header(("Location", url))
                .finish()
        }
        None => {
            // Bloom false positive (very rare with 1% FPR)
            println!("⚠️ FALSE POSITIVE: {} passed Bloom but not in DB", short_code);
            
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Short code not found".to_string(),
                code: 404,
            })
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("══════════════════════════════════════════════════════════════════════");
    println!("🚀 GENESIS Engine v1.0.0");
    println!("══════════════════════════════════════════════════════════════════════");
    println!("🔥 High-Performance Mode: ACTIVATED");
    println!("🧬 L2 Bloom Filter: ONLINE (1M capacity, 1% FPR)");
    println!("💾 L3 In-Memory DB: ONLINE");
    println!("🌐 Server: http://127.0.0.1:8080");
    println!("══════════════════════════════════════════════════════════════════════");

    // Initialize shared state
    let bloom_store = Arc::new(BloomStore::new());
    let db: HashMap<String, String> = HashMap::new();
    
    HttpServer::new(move || {
        let app_state = AppState {
            bloom: Arc::clone(&bloom_store),
            db: Mutex::new(db.clone()),
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
