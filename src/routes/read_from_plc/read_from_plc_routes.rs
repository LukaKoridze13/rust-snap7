use axum::{
    routing::get,
    Router,
};
use crate::routes::AppState;
use super::{read_db_bit, read_db_byte, read_digital_input, read_analog_input};

pub fn read_from_plc_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/read_db_bit", get(read_db_bit))
        .route("/read_db_byte", get(read_db_byte))
        .route("/read_digital_input", get(read_digital_input))
        .route("/read_analog_input", get(read_analog_input))
        .with_state(app_state)
}
