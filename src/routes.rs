use axum::{Router, http::Method};
use tower_http::cors::{Any, CorsLayer};

mod health_check;
use health_check::health_check_routes;

pub fn create_routes() -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    Router::new()
        .merge(health_check_routes())
        .layer(cors)
}
