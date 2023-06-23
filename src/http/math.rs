use serde::Deserialize;
use axum::{
    Router,
    http::StatusCode,
    extract::{State, Json},
    routing::post,
    response::{Html, IntoResponse},
};
use sqlx::PgPool;



#[derive(Deserialize)]
struct MathRequest {
    x: f64,
    y: f64,
}


async fn divide_handler(State(pool): State<PgPool>, Json(payload): Json<MathRequest>) -> impl IntoResponse {
    let x = payload.x;
    let y = payload.y;
    println!("/api/math/divide - {x}, {y}");

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


pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/api/math/divide", post(divide_handler))
        .with_state(pool)
}

