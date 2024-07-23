use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
};
use snap7_rs::S7Client;

pub async fn plc_connection_check() -> Response {
    let client = S7Client::create();
    match client.connect_to("192.168.0.1", 0, 2) {
        Ok(_) => (
            StatusCode::OK,
            "Connected to PLC".to_string()
        ).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Error connecting to PLC: {:?}", e)
        ).into_response(),
    }
}
