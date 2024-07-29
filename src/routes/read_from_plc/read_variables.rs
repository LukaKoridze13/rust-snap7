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
    values: Vec<u8>,
    message: String,
}

pub async fn read_variables(Query(query): Query<ReadVariablesQuery>) -> impl IntoResponse {
    let client = S7Client::create();
    let connection_result = client.connect_to(&query.address, query.rack, query.slot);

    let (status_code, message, values) = match connection_result {
        Ok(_) => {
            let mut buff = vec![0u8; query.size as usize];
            match client.db_read(query.db_number, query.start, query.size, &mut buff) {
                Ok(_) => (
                    StatusCode::OK,
                    "Read successful".to_string(),
                    buff,
                ),
                Err(e) => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("Error reading DB: {:?}", e),
                    vec![],
                ),
            }
        },
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Error connecting to PLC: {:?}", e),
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
        values,
        message,
    };

    (status_code, Json(response)).into_response()
}
