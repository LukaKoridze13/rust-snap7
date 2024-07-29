use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use snap7_rs::S7Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReadVariablesQuery {
    address: String,
    rack: i32,
    slot: i32,
    db_number: i32,
    start: i32,
    size: i32,
}

#[derive(Serialize)]
pub struct ReadVariablesResponse {
    address: String,
    rack: i32,
    slot: i32,
    db_number: i32,
    start: i32,
    size: i32,
    value: Option<u32>,
    bits: Vec<u8>,
    message: String,
}

pub async fn read_variables(Query(query): Query<ReadVariablesQuery>) -> impl IntoResponse {
    let client = S7Client::create();
    let connection_result = client.connect_to(&query.address, query.rack, query.slot);

    let (status_code, message, value, bits) = match connection_result {
        Ok(_) => {
            let mut buff = vec![0u8; query.size as usize];
            match client.db_read(query.db_number, query.start, query.size, &mut buff) {
                Ok(_) => {
                    let mut all_bits = Vec::new();
                    let mut combined_value: Option<u32> = None;

                    // Collect bits from each byte
                    for byte in buff.iter().rev() {
                        all_bits.extend((0..8).map(|bit| ((byte & (1 << bit)) >> bit) as u8));
                    }

                    if query.size == 1 {
                        combined_value = Some(buff[0] as u32);
                    } else if query.size == 2 {
                        combined_value = Some(u16::from_le_bytes([buff[1], buff[0]]) as u32);
                    } else if query.size == 4 {
                        combined_value = Some(u32::from_le_bytes([buff[3], buff[2], buff[1], buff[0]]));
                    }

                    (
                        StatusCode::OK,
                        "Read successful".to_string(),
                        combined_value,
                        all_bits,
                    )
                },
                Err(e) => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("Error reading DB: {:?}", e),
                    None,
                    vec![],
                ),
            }
        },
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Error connecting to PLC: {:?}", e),
            None,
            vec![],
        ),
    };

    let response = ReadVariablesResponse {
        address: query.address.clone(),
        rack: query.rack,
        slot: query.slot,
        db_number: query.db_number,
        start: query.start,
        size: query.size,
        value,
        bits,
        message,
    };

    (status_code, Json(response)).into_response()
}
