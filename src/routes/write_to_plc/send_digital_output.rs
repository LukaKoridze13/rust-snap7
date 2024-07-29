use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use crate::routes::AppState;
use snap7_rs::WordLenTable;

#[derive(Serialize, Deserialize)]
pub struct SendOutputRequest {
    start: i32,
    bit_index: i32, // Index of the bit to set
    value: bool,    // Value to set (true or false)
}

#[derive(Serialize)]
pub struct SendOutputResponse {
    start: i32,
    bit_index: i32,
    value: bool,
    message: String,
}

pub async fn send_digital_output(
    Query(query): Query<SendOutputRequest>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut client_lock = state.client.lock().unwrap();

    // Check if the client is connected
    if client_lock.is_none() {
        let response = SendOutputResponse {
            start: query.start,
            bit_index: query.bit_index,
            value: query.value,
            message: "PLC connection required. Please connect to the PLC first.".to_string(),
        };
        return (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response();
    }

    let client_ref = client_lock.as_mut().unwrap();
    let mut byte_buff = [0u8; 1];

    // Read the current byte value from the PLC
    let read_result = client_ref.read_area(
        snap7_rs::AreaTable::S7AreaPA, // Adjust if needed
        0, // DB number - adjust if needed
        query.start,
        1, // Reading 1 byte
        WordLenTable::S7WLByte,
        &mut byte_buff
    );

    if let Err(e) = read_result {
        let message = if e.to_string().contains("Connection timed out") {
            "PLC connection required. Please connect to the PLC first.".to_string()
        } else {
            format!("Error reading from PLC: {:?}", e)
        };

        let response = SendOutputResponse {
            start: query.start,
            bit_index: query.bit_index,
            value: query.value,
            message,
        };

        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(response),
        ).into_response();
    }

    // Modify the specific bit
    if query.value {
        byte_buff[0] |= 1 << query.bit_index; // Set the bit
    } else {
        byte_buff[0] &= !(1 << query.bit_index); // Clear the bit
    }

    // Write back to the PLC
    let write_result = client_ref.write_area(
        snap7_rs::AreaTable::S7AreaPA, // Adjust if needed
        0, // DB number - adjust if needed
        query.start,
        1, // Writing 1 byte
        WordLenTable::S7WLByte,
        & mut byte_buff // Write the modified byte
    );

    if let Err(e) = write_result {
        let message = if e.to_string().contains("Connection timed out") {
            "PLC connection required. Please connect to the PLC first.".to_string()
        } else {
            format!("Error writing to PLC: {:?}", e)
        };

        let response = SendOutputResponse {
            start: query.start,
            bit_index: query.bit_index,
            value: query.value,
            message,
        };

        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(response),
        ).into_response();
    }

    let response = SendOutputResponse {
        start: query.start,
        bit_index: query.bit_index,
        value: query.value,
        message: "Output successfully set.".to_string(),
    };

    (StatusCode::OK, Json(response)).into_response()
}
