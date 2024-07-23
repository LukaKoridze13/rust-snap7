use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use snap7_rs::S7Client;
use serde::{Deserialize, Serialize};

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

pub async fn plc_connection_check(Query(query): Query<PLCConnectionCheckQuery>) -> impl IntoResponse {
    let client = S7Client::create();
    let connection_result = client.connect_to(&query.address, query.rack, query.slot);

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

    let response = PLCConnectionCheckResponse {
        address: query.address.clone(),
        rack: query.rack,
        slot: query.slot,
        message,
    };

    (status_code, Json(response)).into_response()
}
