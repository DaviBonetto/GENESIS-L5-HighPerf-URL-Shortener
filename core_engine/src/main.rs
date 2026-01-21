//! GENESIS Engine v1.0.0 - Production Grade URL Shortener
//!
//! Part of the Titan Protocol Initiative.
//!
//! Architecture:
//! ┌─────────────────────────────────────────────────────────┐
//! │ L1: Client Cache (Cache-Control headers)                │
//! ├─────────────────────────────────────────────────────────┤
//! │ L2: Bloom Filter (O(1) probabilistic existence check)   │
//! ├─────────────────────────────────────────────────────────┤
//! │ L4: PostgreSQL (persistent storage)                     │
//! └─────────────────────────────────────────────────────────┘

mod storage;
mod utils;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;

use storage::BloomStore;
use utils::base62;

/// Application state shared across all requests
struct AppState {
    bloom: Arc<BloomStore>,
    db: PgPool,
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

/// Health check endpoint with system stats
#[get("/health")]
async fn health_check(data: web::Data<AppState>) -> impl Responder {
    let memory_kb = data.bloom.memory_usage() / 1024;

    // Check database connectivity
    let db_status = match sqlx::query("SELECT 1").fetch_one(&data.db).await {
        Ok(_) => "CONNECTED",
        Err(_) => "DISCONNECTED",
    };

    HttpResponse::Ok().body(format!(
        "Genesis Engine v1.0.0: OPERATIONAL 🟢\n\
         Bloom Filter Memory: {} KB\n\
         PostgreSQL: {}",
        memory_kb, db_status
    ))
}

/// Create a shortened URL
///
/// Flow:
/// 1. Generate random u64 -> Base62 short code
/// 2. Store in Bloom Filter (L2)
/// 3. Store in PostgreSQL (L4)
/// 4. Return short URL
#[post("/shorten")]
async fn shorten_url(
    data: web::Data<AppState>,
    body: web::Json<CreateLinkRequest>,
) -> impl Responder {
    // Generate random ID and encode to Base62
    let random_id: u64 = rand::random();
    let short_code = base62::encode(random_id);

    // L2: Store in Bloom Filter
    data.bloom.add(&short_code);

    // L4: Store in PostgreSQL
    let result = sqlx::query("INSERT INTO urls (id, original_url) VALUES ($1, $2)")
        .bind(&short_code)
        .bind(&body.url)
        .execute(&data.db)
        .await;

    match result {
        Ok(_) => {
            let short_url = format!("http://127.0.0.1:8080/{}", short_code);
            println!("✨ CREATED: {} -> {}", short_code, body.url);

            HttpResponse::Created().json(CreateLinkResponse {
                short_url,
                original_url: body.url.clone(),
                short_code,
            })
        }
        Err(e) => {
            println!("❌ DATABASE ERROR: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create short URL".to_string(),
                code: 500,
            })
        }
    }
}

/// Resolve a short code to its original URL
///
/// Flow:
/// 1. L2 Bloom Filter check (O(1)) - blocks non-existent codes
/// 2. L4 PostgreSQL lookup - retrieves original URL
/// 3. HTTP 302 redirect with L1 Cache-Control header
#[get("/{short_code}")]
async fn resolve_url(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
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

    // L4: PostgreSQL Lookup
    let result = sqlx::query_scalar::<_, String>("SELECT original_url FROM urls WHERE id = $1")
        .bind(&short_code)
        .fetch_optional(&data.db)
        .await;

    match result {
        Ok(Some(original_url)) => {
            println!("✅ REDIRECT: {} -> {}", short_code, original_url);

            // HTTP 302 Found with L1 Cache-Control
            HttpResponse::Found()
                .append_header(("Location", original_url))
                .append_header(("Cache-Control", "public, max-age=3600"))
                .finish()
        }
        Ok(None) => {
            // Bloom false positive (very rare with 1% FPR)
            println!(
                "⚠️ FALSE POSITIVE: {} passed Bloom but not in DB",
                short_code
            );

            HttpResponse::NotFound().json(ErrorResponse {
                error: "Short code not found".to_string(),
                code: 404,
            })
        }
        Err(e) => {
            println!("❌ DATABASE ERROR: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Database error".to_string(),
                code: 500,
            })
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    println!("══════════════════════════════════════════════════════════════════════");
    println!("🚀 GENESIS Engine v1.0.0 - Production Grade");
    println!("══════════════════════════════════════════════════════════════════════");

    // Connect to PostgreSQL
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    println!("📊 Connecting to PostgreSQL...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    println!("   ✅ PostgreSQL connected");

    // Run migrations
    println!("📋 Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    println!("   ✅ Migrations complete");

    // Initialize Bloom Filter
    let bloom_store = Arc::new(BloomStore::new());

    // Preload existing URLs into Bloom Filter
    println!("🧬 Preloading Bloom Filter from database...");
    let existing_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM urls")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    for id in &existing_ids {
        bloom_store.add(id);
    }
    println!(
        "   ✅ Loaded {} existing URLs into Bloom Filter",
        existing_ids.len()
    );

    println!("══════════════════════════════════════════════════════════════════════");
    println!("🔥 High-Performance Mode: ACTIVATED");
    println!("🧬 L2 Bloom Filter: ONLINE (1M capacity, 1% FPR)");
    println!("💾 L4 PostgreSQL: ONLINE");
    println!("🌐 Server: http://127.0.0.1:8080");
    println!("══════════════════════════════════════════════════════════════════════");

    HttpServer::new(move || {
        let app_state = AppState {
            bloom: Arc::clone(&bloom_store),
            db: pool.clone(),
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
