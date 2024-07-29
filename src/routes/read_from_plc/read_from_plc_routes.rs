use axum::{
    routing::get,
    Router,
};

use super::read_variables;


pub fn read_from_plc_routes() -> Router {
    Router::new()
        .route("/read_variables", get(read_variables))
}
