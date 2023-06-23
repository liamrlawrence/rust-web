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
        .merge(http::math::router(pool));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Starting the server on {addr}...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

