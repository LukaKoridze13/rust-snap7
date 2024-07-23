use axum::{
    http::Method,
    routing::get ,
     Router,
};

mod plc_connection_check;
use plc_connection_check::plc_connection_check;

use tower_http::cors::{Any, CorsLayer};



pub fn create_routes() -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    Router::new()
        .route("/plc_connection_check", get(plc_connection_check))
        .layer(cors)
}
