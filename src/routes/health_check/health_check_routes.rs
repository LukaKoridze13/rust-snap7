use axum::{routing::get, Router};

use super::{plc_connection_check, server_health_check};

pub fn health_check_routes() -> Router {
    Router::new()
        .route("/server_health_check", get(server_health_check))
        .route("/plc_connection_check", get(plc_connection_check))
}
