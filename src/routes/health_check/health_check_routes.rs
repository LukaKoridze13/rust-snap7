use axum::{routing::get, Router};
use crate::routes::AppState;

use super::{plc_connection_check, server_health_check};

pub fn health_check_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/server_health_check", get(server_health_check))
        .route("/plc_connection_check", get(plc_connection_check))
        .with_state(app_state)
}
