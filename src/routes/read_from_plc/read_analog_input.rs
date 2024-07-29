use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use crate::routes::AppState;
use snap7_rs::WordLenTable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReadAnalogInputQuery {
    byte_index: i32,
    size: i32, // Size of data to read in bytes
}

#[derive(Serialize)]
pub struct ReadAnalogInputResponse {
    byte_index: i32,
    size: i32,
    value: Option<u32>, // Value interpreted as u32
    message: String,
}

pub async fn read_analog_input(
    Query(query): Query<ReadAnalogInputQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut client_lock = state.client.lock().unwrap();

    if client_lock.is_none() {
        let response = ReadAnalogInputResponse {
            byte_index: query.byte_index,
            size: query.size,
            value: None,
            message: "PLC connection required. Please connect to the PLC first.".to_string(),
        };
        return (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response();
    }

    let client_ref = client_lock.as_mut().unwrap();
    let mut byte_buff = vec![0u8; query.size as usize];

    let (status_code, message, value) = match client_ref.read_area(
        snap7_rs::AreaTable::S7AreaPE, // Using the Analog Input Area
        0, // DB number is ignored for input area
        query.byte_index,
        query.size, // Reading multiple bytes based on size
        WordLenTable::S7WLByte, // Reading bytes
        &mut byte_buff
    ) {
        Ok(_) => {
            let mut combined_value: Option<u32> = None;

            // Combine bytes into a single u32 value
            if query.size == 1 {
                combined_value = Some(byte_buff[0] as u32);
            } else if query.size == 2 {
                combined_value = Some(u16::from_le_bytes([byte_buff[1], byte_buff[0]]) as u32);
            } else if query.size == 4 {
                combined_value = Some(u32::from_le_bytes([byte_buff[3], byte_buff[2], byte_buff[1], byte_buff[0]]));
            }

            (
                StatusCode::OK,
                "Read successful".to_string(),
                combined_value,
            )
        },
        Err(e) => {
            let message = if e.to_string().contains("Connection timed out") {
                "PLC connection required. Please connect to the PLC first.".to_string()
            } else {
                format!("Error reading analog input: {:?}", e)
            };

            (
                StatusCode::SERVICE_UNAVAILABLE,
                message,
                None,
            )
        }
    };

    let response = ReadAnalogInputResponse {
        byte_index: query.byte_index,
        size: query.size,
        value,
        message,
    };

    (status_code, Json(response)).into_response()
}
