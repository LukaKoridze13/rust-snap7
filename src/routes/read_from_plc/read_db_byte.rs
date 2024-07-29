use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use crate::routes::AppState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReadVariablesQuery {
    db_number: i32,
    start: i32,
    size: i32,
    data_type: String, // Added data_type field to query
}

#[derive(Serialize)]
pub struct ReadVariablesResponse {
    db_number: i32,
    start: i32,
    size: i32,
    value: Option<f32>, // Changed value type to f32 to handle floating points
    bits: String, // Changed type to String
    message: String,
}

pub async fn read_db_byte(
    Query(query): Query<ReadVariablesQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut client_lock = state.client.lock().unwrap();

    if client_lock.is_none() {
        let response = ReadVariablesResponse {
            db_number: query.db_number,
            start: query.start,
            size: query.size,
            value: None,
            bits: String::new(),
            message: "PLC connection required. Please connect to the PLC first.".to_string(),
        };
        return (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response();
    }

    let client_ref = client_lock.as_mut().unwrap();
    let mut buff = vec![0u8; query.size as usize];
    
    let (status_code, message, value, bits) = match client_ref.db_read(query.db_number, query.start, query.size, &mut buff) {
        Ok(_) => {
            let mut all_bits = Vec::new();
            let mut combined_value: Option<f32> = None;

            // Collect bits from each byte
            for byte in buff.iter() {
                all_bits.extend((0..8).map(|bit| ((byte & (1 << bit)) >> bit) as u8));
            }

            // Reverse the bits and convert to a string
            let bits_string: String = all_bits.iter().map(|&bit| if bit == 1 { '1' } else { '0' }).collect();

            if query.data_type == "integer" {
                if query.size == 1 {
                    combined_value = Some(buff[0] as u32 as f32);
                } else if query.size == 2 {
                    combined_value = Some(u16::from_le_bytes([buff[1], buff[0]]) as u32 as f32);
                } else if query.size == 4 {
                    combined_value = Some(u32::from_le_bytes([buff[3], buff[2], buff[1], buff[0]]) as f32);
                }
            } else if query.data_type == "floating_point" {
                if query.size == 4 {
                    let value_as_integer = u32::from_le_bytes([buff[3], buff[2], buff[1], buff[0]]);
                    combined_value = Some(f32::from_bits(value_as_integer));
                }
            }

            (
                StatusCode::OK,
                "Read successful".to_string(),
                combined_value,
                bits_string,
            )
        }
        Err(e) => {
            let message = if e.to_string().contains("Connection timed out") {
                "PLC connection required. Please connect to the PLC first.".to_string()
            } else {
                format!("Error reading DB: {:?}", e)
            };

            (
                StatusCode::SERVICE_UNAVAILABLE,
                message,
                None,
                String::new(),
            )
        }
    };

    let response = ReadVariablesResponse {
        db_number: query.db_number,
        start: query.start,
        size: query.size,
        value,
        bits,
        message,
    };

    (status_code, Json(response)).into_response()
}
