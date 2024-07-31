use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{heater::Heater, routes::AppState};
use pid::Pid;

pub struct SharedState {
    pub app_state: Arc<Mutex<AppState>>,
    pub heater: Heater,
    pub pid: Arc<Mutex<Pid<f64>>>,
}


pub async fn enable_heater(State(state): State<Arc<Mutex<SharedState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    state.heater.enable(&state.app_state, &state.pid).await;
    (StatusCode::OK, Json("Heater enabled".to_string()))
}

pub async fn disable_heater(State(state): State<Arc<Mutex<SharedState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    state.heater.disable().await;
    (StatusCode::OK, Json("Heater disabled".to_string()))
}
