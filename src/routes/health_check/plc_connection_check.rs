use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use snap7_rs::S7Client;
use crate::routes::AppState;

#[derive(Serialize, Deserialize)]
pub struct PLCConnectionCheckQuery {
    address: String,
    rack: i32,
    slot: i32,
}

#[derive(Serialize)]
pub struct PLCConnectionCheckResponse {
    address: String,
    rack: i32,
    slot: i32,
    message: String,
}

pub async fn plc_connection_check(
    Query(query): Query<PLCConnectionCheckQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut client_lock = state.client.lock().unwrap();

    // Drop the existing client if present
    if client_lock.is_some() {
        // Optionally, handle disconnection here if needed
        // E.g., client_lock.as_mut().unwrap().disconnect();
        *client_lock = None;
    }

    // Create a new client and connect to the PLC
    let client = S7Client::create();
    let connection_result = client.connect_to(&query.address, query.rack, query.slot);

    let (status_code, message) = match connection_result {
        Ok(_) => {
            *client_lock = Some(client);
            (
                StatusCode::OK,
                "Connected to PLC".to_string(),
            )
        }
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Error connecting to PLC: {:?}", e),
        ),
    };

    let response = PLCConnectionCheckResponse {
        address: query.address.clone(),
        rack: query.rack,
        slot: query.slot,
        message,
    };

    (status_code, Json(response)).into_response()
}
