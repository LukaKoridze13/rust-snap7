use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::{Arc, Mutex};

use crate::routes::{AppState, PLCConfig};

#[derive(Serialize)]
struct PlcStatusResponse {
    status_code: i32,
    message: String,
}

pub async fn get_plc_operating_mode(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    // Lock the state to get access
    let state = state.lock().unwrap();

    match state.get_plc_status() {
        Ok(status) => {
            let message = match status {
                0x00 => "Status Unknown".to_string(),
                0x08 => "Running".to_string(),
                0x04 => "Stopped".to_string(),
                _ => "Unknown Status Code".to_string(),
            };
            (StatusCode::OK, Json(PlcStatusResponse { status_code: status, message }))
        },
        Err(e) => {
            let error_message = format!("Failed to get PLC status: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(PlcStatusResponse { status_code: -1, message: error_message }))
        },
    }
}

#[derive(Serialize)]
struct ChangeConnectionResponse {
    message: String,
}

pub async fn change_plc_connection_settings(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(new_config): Json<PLCConfig>,
) -> impl IntoResponse {

    // Lock the state to get mutable access
    let mut state = state.lock().unwrap();

    // Update the configuration
    state.update_config(new_config);

    // Attempt to disconnect
    let disconnect_result = state.s7_client.disconnect();
    match disconnect_result {
        Ok(_) => (),
        Err(e) => {
            println!("** Failed to disconnect from PLC: {:?}", e);
        }
    };


    // Attempt to reconnect
    let connection_result = state.connect_to_plc();

    let (status_code, response) = match connection_result {
        Ok(_) => (
            StatusCode::OK,
            Json(ChangeConnectionResponse {
                message: "Config updated and connected to PLC".to_string(),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ChangeConnectionResponse {
                message: format!("Config updated but can't connect to PLC. Reason: {:?}", e),
            }),
        ),
    };

    (status_code, response)
}

pub async fn stop_plc(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    // Lock the state to get access
    let state = state.lock().unwrap();
    
    // Attempt to stop the PLC
    let result = state.s7_client.plc_stop();
    
    // Determine the response based on the result
    let (status_code, response) = match result {
        Ok(_) => (
            StatusCode::OK,
            Json(ChangeConnectionResponse {
                message: "PLC Stopped".to_string(),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ChangeConnectionResponse {
                message: format!("Couldn't stop PLC. Reason: {:?}", e),
            }),
        ),
    };
    
    (status_code, response)
}

pub async fn hot_start(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    // Lock the state to get access
    let  state = state.lock().unwrap();

    // Check the current PLC status
    let status = match state.get_plc_status() {
        Ok(status) => status,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChangeConnectionResponse {
                    message: format!("Failed to get PLC status: {:?}", e),
                }),
            ).into_response();
        }
    };

    // If the PLC is already running, return an appropriate message
    if status == 0x08 {
        return (
            StatusCode::OK,
            Json(ChangeConnectionResponse {
                message: "PLC is already running".to_string(),
            }),
        ).into_response();
    }
    
    // Attempt to start the PLC in hot mode
    let result = state.s7_client.plc_hot_start();
    
    // Determine the response based on the result
    let (status_code, response) = match result {
        Ok(_) => (
            StatusCode::OK,
            Json(ChangeConnectionResponse {
                message: "PLC Started - Mode: HOT".to_string(),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ChangeConnectionResponse {
                message: format!("Couldn't start PLC. Reason: {:?}", e),
            }),
        ),
    };
    
    (status_code, response).into_response()
}

pub async fn cold_start(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let  state = state.lock().unwrap();

    // Check the current PLC status
    let status = match state.get_plc_status() {
        Ok(status) => status,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChangeConnectionResponse {
                    message: format!("Failed to get PLC status: {:?}", e),
                }),
            ).into_response();
        }
    };

    // If the PLC is already running, return an appropriate message
    if status == 0x08 {
        return (
            StatusCode::OK,
            Json(ChangeConnectionResponse {
                message: "PLC is already running".to_string(),
            }),
        ).into_response();
    }
    
    // Attempt to stop the PLC
    let result = state.s7_client.plc_cold_start();
    
    // Determine the response based on the result
    let (status_code, response) = match result {
        Ok(_) => (
            StatusCode::OK,
            Json(ChangeConnectionResponse {
                message: "PLC Started - Mode: Cold".to_string(),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ChangeConnectionResponse {
                message: format!("Couldn't start PLC. Reason: {:?}", e),
            }),
        ),
    };
    
    (status_code, response).into_response()
}