use serde::{Serialize, Deserialize};
use std::net::IpAddr;
use axum::{
    Router,
    http::{StatusCode, HeaderMap, header::HeaderValue, header::FORWARDED, request::Parts, Response},
    extract::{State, Json, FromRequestParts},
    routing::post,
    response::{Html, IntoResponse}, async_trait,
    TypedHeader,
    headers::authorization::{Authorization, Bearer},
};
use axum_extra::extract::cookie::{CookieJar, Cookie};
use sqlx::PgPool;
use sqlx::types::{ipnetwork::IpNetwork, Uuid};




pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/refresh", post(refresh_token_handler))
        .with_state(pool)
}


// TODO: Is this correct?
struct ExtractHeaderForwarded(HeaderValue);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractHeaderForwarded
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(forwarded) = parts.headers.get(FORWARDED) {
            Ok(ExtractHeaderForwarded(forwarded.clone()))
        } else {
            Err((StatusCode::BAD_REQUEST, "`Forwarded` header is missing"))
        }
    }
}



#[derive(Deserialize)]
struct LoginRequest {
   username: String,
   password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    status: u16,
    message: String,
}

async fn login_handler(
    State(pool): State<PgPool>,
    cookie_jar: CookieJar,
    ExtractHeaderForwarded(ip_address): ExtractHeaderForwarded,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let ip_string = &ip_address.to_str().unwrap()[4..];
    let ip_addr: IpNetwork = ip_string.parse().unwrap();

    let row = sqlx::query!(r"
            SELECT *
            FROM Auth.FN_User_Login($1::TEXT, $2::TEXT, $3::INET);
        ", payload.username, payload.password, ip_addr)
        .fetch_one(&pool)
        .await.unwrap();

    let new_session_token: String = row.session_id.unwrap();
    let new_refresh_token: String = row.refresh_token.unwrap();

    let response = LoginResponse {
        status: 200,
        message: "Successfully logged in".to_string(),
    };


    return(
        StatusCode::OK,
        cookie_jar
            .add(Cookie::build("X-Session-Token", new_session_token)
                 .path("/api/")
                 .secure(true)
                 .http_only(true)
                 .finish())
            .add(Cookie::build("X-Refresh-Token", new_refresh_token)
                 .path("/api/")
                 .secure(true)
                 .http_only(true)
                 .finish()),
        Json(response)
    )
}



#[derive(Deserialize)]
struct RefreshRequest {
   refresh_token: String,
}

#[derive(Serialize)]
struct RefreshResponse {
    status: u16,
    message: String,
}

async fn refresh_token_handler(
    State(pool): State<PgPool>,
    cookie_jar: CookieJar,
    ExtractHeaderForwarded(ip_address): ExtractHeaderForwarded,
    TypedHeader(session_token): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse {
    let ip_string = &ip_address.to_str().unwrap()[4..];
    let ip_addr: IpNetwork = ip_string.parse().unwrap();

    let row = sqlx::query!(r"
            SELECT *
            FROM Auth.FN_Refresh_Session($1::UUID, $2::UUID, $3::INET);
        ", Uuid::parse_str(session_token.token()).unwrap(), Uuid::parse_str(&payload.refresh_token).unwrap(), ip_addr)
        .fetch_one(&pool)
        .await.unwrap();

    let refresh_uuid: Uuid = row.fn_refresh_session.unwrap();
    let new_refresh_token: String = refresh_uuid.to_string();

    let response = RefreshResponse {
        status: 200,
        message: "Refresh token generated".to_string(),
    };

    return(
        StatusCode::OK,
        cookie_jar
            .add(Cookie::build("X-Refresh-Token", new_refresh_token)
                 .path("/api/")
                 .secure(true)
                 .http_only(true)
                 .finish()),
        Json(response)
    )
}



async fn signup_handler() {

}

async fn logout_handler() {

}

async fn update_password_handler() {

}

