use std::env;
use std::time::Duration;
use std::net::SocketAddr;
use serde::Deserialize;
use axum::{
    Router,
    http::StatusCode,
    extract::{State, Json},
    routing::post,
    response::{Html, IntoResponse},
};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;



#[derive(Deserialize)]
struct MathRequest {
    x: f64,
    y: f64,
}


async fn divide_handler(State(pool): State<PgPool>, Json(payload): Json<MathRequest>) -> impl IntoResponse {
    let x = payload.x;
    let y = payload.y;

    match perform_division(pool, x, y).await {
        Ok(result) => (
            StatusCode::OK,
            Html(format!("<p>Result: {}</p>", result))
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Html(format!("<p>Error: {}</p>", e))
        ),
    }
}


async fn perform_division(pool: PgPool, x: f64, y: f64) -> Result<f64, String> {
    let row: (f64,) = sqlx::query_as("SELECT divide_xy($1, $2);")
        .bind(x)
        .bind(y)
        .fetch_one(&pool)
        .await
        .unwrap();

    Ok(row.0)
}



#[tokio::main]
async fn main() {
    // Connect to the database
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&format!("postgres://{}:{}@{}:{}/{}",
            env::var("PSQL_DB_USERNAME").unwrap(),
            env::var("PSQL_DB_PASSWORD").unwrap(),
            env::var("PSQL_DB_HOST").unwrap(),
            env::var("PSQL_DB_PORT").unwrap(),
            env::var("PSQL_DB_DATABASE").unwrap()))
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        .route("/api/math/divide", post(divide_handler))
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Starting the server on {addr}...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

