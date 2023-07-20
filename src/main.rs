use std::time::Duration;
use std::net::SocketAddr;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{CorsLayer};

mod http;



#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_origin("http://localhost:3000".parse::<axum::http::HeaderValue>().unwrap())
        .allow_headers([axum::http::header::CONTENT_TYPE]);

        //.allow_origin("http://localhost:3000".parse::<axum::http::HeaderValue>().unwrap())
        //.allow_headers([axum::http::header::CONTENT_TYPE])
        //.allow_methods([axum::http::Method::GET, axum::http::Method::POST]);
        //.max_age(Duration::from_secs(60 * 60));

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        .nest("/api/ai",    http::ai::router(pool.clone()))
        .nest("/api/auth",  http::auth::router(pool.clone()))
        .layer(cors)
        .nest("/api/math",  http::math::router(pool.clone()));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Starting the server on {addr}...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

