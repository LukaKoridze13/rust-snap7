use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use crate::routes::AppState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReadBitQuery {
    db_number: i32,
    start: i32,
    bit: u8,
}

#[derive(Serialize)]
pub struct ReadBitResponse {
    db_number: i32,
    start: i32,
    bit: u8,
    value: Option<u8>,
    message: String,
}

pub async fn read_db_bit(
    Query(query): Query<ReadBitQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut client = state.client.lock().unwrap();
    
    // Check if client is connected
    if client.is_none() {
        let response = ReadBitResponse {
            db_number: query.db_number,
            start: query.start,
            bit: query.bit,
            value: None,
            message: "PLC connection required. Please connect to the PLC first.".to_string(),
        };
        return (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response();
    }

    // Client is connected, proceed with reading
    let client_ref = client.as_mut().unwrap();
    let mut buff = vec![0u8; 1];
    match client_ref.db_read(query.db_number, query.start, 1, &mut buff) {
        Ok(_) => {
            let byte = buff[0];
            let bit_value = (byte >> query.bit & 1) as u8;

            let response = ReadBitResponse {
                db_number: query.db_number,
                start: query.start,
                bit: query.bit,
                value: Some(bit_value),
                message: "Read successful".to_string(),
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            // Check if the error is related to connection issues
            let message = if e.to_string().contains("Connection timed out") {
                "PLC connection required. Please connect to the PLC first.".to_string()
            } else {
                format!("Error reading DB: {:?}", e)
            };

            let response = ReadBitResponse {
                db_number: query.db_number,
                start: query.start,
                bit: query.bit,
                value: None,
                message,
            };

            (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response()
        }
    }
}
