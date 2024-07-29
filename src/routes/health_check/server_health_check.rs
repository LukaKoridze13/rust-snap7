use axum::{
    http::StatusCode,
    response::IntoResponse
};

pub async fn server_health_check() -> impl IntoResponse {
    (StatusCode::OK, "Server Status Is OK".to_string()).into_response()
}
