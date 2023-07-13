use std::collections::HashMap;
use futures::TryStreamExt;

use reqwest::header;
use serde::{Serialize, Deserialize};
use axum::{
    Router,
    http::{StatusCode, Request, Uri},
    extract::{State, Json, FromRequestParts},
    routing::{post, get},
    response::{Html, IntoResponse}, async_trait, TypedHeader,
    headers::authorization::{Authorization, Bearer},
};
use serde_json::Value;
use sqlx::{PgPool, Row};
use sqlx::types::Uuid;



pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/gpt2", post(chat_gpt_handler))
        .route("/gpt3", post(chat_gpt_handler))
        .route("/gpt4", post(chat_gpt_handler))
        .route("/bills", get(ai_bills_handler)) 
        .with_state(pool)
}


#[derive(Deserialize)]
struct ChatGptRequest {
    name: String,
    prompt: String,
}

#[derive(Serialize)]
struct ChatGptResponse {
    status: i32,
    message: String,
    data: Option<OpenAiResponse>,
}

#[derive(Serialize)]
struct OpenAiRequestMessage {
    content: String,
    role: String,
}

#[derive(Serialize)]
struct OpenAiRequestData {
    model: String,
    messages: Vec<OpenAiRequestMessage>,
    temperature: f64,
}

#[derive(Serialize)]
struct OpenAiRequest {
    status: i32,
    messages: Vec<OpenAiRequestMessage>,
    data: OpenAiRequestData,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    content: String,
    role: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiChoice {
    finish_reason: String,
    index: i32,
    message: OpenAiMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiUsage {
    completion_tokens: i32,
    prompt_tokens: i32,
    total_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    created: f64,
    id: String,
    model: String,
    object: String,
    usage: OpenAiUsage
}


async fn chat_gpt_handler(
    State(pool): State<PgPool>,
    uri: Uri,
    TypedHeader(session_token): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<ChatGptRequest>,
) -> impl IntoResponse {
    // Make API request to OpenAI
    let openai_api_url = "https://api.openai.com/v1/chat/completions";
    let openai_key = std::env::var("OPENAI_API_KEY").unwrap();

    let model = match uri.path() {
        "/gpt3" => "gpt-3.5-turbo",
        "/gpt4" => "gpt-4-0613",
        &_ => {
            return(
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatGptResponse {
                    status: 200,
                    message: "Success!".to_string(),
                    data: None,
                })
            )
        }
    };

    let body = &serde_json::json!(OpenAiRequestData {
        model: model.to_string(),
        messages: vec![OpenAiRequestMessage {
            role: "user".to_string(),
            content: payload.prompt.to_string(),
        }],
        temperature: 0.7
    });

    let res: OpenAiResponse = reqwest::Client::new()
        .post(openai_api_url)
        .header(header::AUTHORIZATION, format!("Bearer {}", openai_key))
        .header(header::CONTENT_TYPE, "application/json")
        .json(body)
        .send()
        .await.unwrap()
        .json::<OpenAiResponse>()
        .await.unwrap();


    // Log bill
    let row = sqlx::query!(r"
            CALL SP_Insert_GPT_Bill($1::UUID, $2::TEXT, $3::TEXT, $4::INT, $5::INT); 
        ", Uuid::parse_str(session_token.token()).unwrap(), payload.name.to_string(), model.to_string(), res.usage.prompt_tokens, res.usage.completion_tokens)
        .execute(&pool)
        .await.unwrap();


    return(
        StatusCode::OK,
        Json(ChatGptResponse {
            status: 200,
            message: "Success!".to_string(),
            data: Some(res)
        }),
    )
}




#[derive(Serialize)]
struct AiBillsResponseData {
    billed_to: Vec<String>,
    model: Vec<String>,
    prompt_tokens: Vec<i64>,
    completion_tokens: Vec<i64>,
    total_cost: Vec<f64>,
}

#[derive(Serialize)]
struct AiBillsResponse {
    status: i32,
    message: String,
    data: AiBillsResponseData,
}

async fn ai_bills_handler(
    State(pool): State<PgPool>,
) -> impl IntoResponse {

    let rows = sqlx::query(r"
            SELECT
                billed_to,
                model,
                prompt_tokens,
                completion_tokens,
                total_cost::FLOAT8
            FROM View_AI_Bills;
        ")
        .fetch_all(&pool)
        .await.unwrap();

    let mut response = AiBillsResponse {
        status: 200,
        message: "Success!".to_string(),
        data: AiBillsResponseData {
            billed_to: Vec::new(),
            model: Vec::new(),
            prompt_tokens: Vec::new(),
            completion_tokens: Vec::new(),
            total_cost: Vec::new()
        }
    };

    for row in rows {
        let billed_to: String = row.try_get("billed_to").unwrap();
        let model: String = row.try_get("model").unwrap();
        let prompt_tokens: i64 = row.try_get("prompt_tokens").unwrap();
        let completion_tokens: i64 = row.try_get("completion_tokens").unwrap();
        let total_cost: f64 = row.try_get("total_cost").unwrap();

        response.data.billed_to.push(billed_to);
        response.data.model.push(model);
        response.data.prompt_tokens.push(prompt_tokens);
        response.data.completion_tokens.push(completion_tokens);
        response.data.total_cost.push(total_cost);
    }

    return(
        StatusCode::OK,
        Json(response),
    )
}

