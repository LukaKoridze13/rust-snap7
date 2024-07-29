use axum::{
    routing::post,
    Router,
};
use crate::routes::AppState;
use super::send_digital_output;

pub fn write_to_plc_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/send_digital_output", post(send_digital_output))
        .with_state(app_state)
}
