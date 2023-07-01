use std::time::Duration;
use std::net::SocketAddr;
use axum::Router;
use sqlx::postgres::PgPoolOptions;

mod http;



#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        .nest("/api/auth", http::auth::router(pool.clone()))
        .nest("/api/math", http::math::router(pool.clone()))
        .nest("/api/ai", http::ai::router(pool.clone()));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Starting the server on {addr}...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

