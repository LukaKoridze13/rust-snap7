use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use crate::routes::AppState;
use snap7_rs::WordLenTable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReadDigitalInputQuery {
    byte_index: i32,
    bit_index: i32,
}

#[derive(Serialize)]
pub struct ReadDigitalInputResponse {
    byte_index: i32,
    bit_index: i32,
    value: Option<u8>,
    message: String,
}

pub async fn read_digital_input(
    Query(query): Query<ReadDigitalInputQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut client_lock = state.client.lock().unwrap();

    if client_lock.is_none() {
        let response = ReadDigitalInputResponse {
            byte_index: query.byte_index,
            bit_index: query.bit_index,
            value: None,
            message: "PLC connection required. Please connect to the PLC first.".to_string(),
        };
        return (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response();
    }

    let client_ref = client_lock.as_mut().unwrap();
    let mut byte_buff = [0u8; 1];
    
    let (status_code, message, value) = match client_ref.read_area(
        snap7_rs::AreaTable::S7AreaPE, // Using the Input Area
        0, // DB number is ignored for input area
        query.byte_index,
        1, // Reading 1 byte
        WordLenTable::S7WLBit,
        &mut byte_buff
    ) {
        Ok(_) => {
            let bit_value = (byte_buff[0] >> query.bit_index) & 1;
            (
                StatusCode::OK,
                "Read successful".to_string(),
                Some(bit_value),
            )
        },
        Err(e) => {
            let message = if e.to_string().contains("Connection timed out") {
                "PLC connection required. Please connect to the PLC first.".to_string()
            } else {
                format!("Error reading input: {:?}", e)
            };

            (
                StatusCode::SERVICE_UNAVAILABLE,
                message,
                None,
            )
        }
    };

    let response = ReadDigitalInputResponse {
        byte_index: query.byte_index,
        bit_index: query.bit_index,
        value,
        message,
    };

    (status_code, Json(response)).into_response()
}
