use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{
    extract::State, http::StatusCode, response::IntoResponse, Json
};
use serde::Serialize;

use super::SharedState;

#[derive(Serialize)]
struct Response {
    message: String,
}

pub async fn server_health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(Response{
        message: "Server is healthy".to_string(),
    }))
}


#[derive(Serialize)]
pub struct PLCResponse {
    address: String,
    rack: i32,
    slot: i32,
    message: String,
}

pub async fn plc_connection_check(
    State(state): State<Arc<Mutex<SharedState>>>,
) -> impl IntoResponse {
    // Lock the state to get a mutable reference
    let state = state.lock().await;
    let app_state = state.app_state.lock().await;
    // Attempt to connect to the PLC
    let connection_result = app_state.connect_to_plc();

    // Determine the response based on the connection result
    let (status_code, message) = match connection_result {
        Ok(_) => (
            StatusCode::OK,
            "Connected to PLC".to_string(),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Error connecting to PLC: {:?}", e),
        ),
    };

    // Create the response struct with the current state information
    let response = PLCResponse {
        address: app_state.address.clone(),
        rack: app_state.rack,
        slot: app_state.slot,
        message,
    };

    // Return the response
    (status_code, Json(response)).into_response()
}
