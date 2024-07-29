use axum::{Router, http::Method};
use tower_http::cors::{Any, CorsLayer};

mod health_check;
mod read_from_plc;

use health_check::health_check_routes;
use read_from_plc::read_from_plc_routes;

pub fn create_routes() -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    Router::new()
        .merge(health_check_routes())
        .merge(read_from_plc_routes())
        .layer(cors)
}
