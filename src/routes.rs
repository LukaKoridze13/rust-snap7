use axum::{Router, http::Method};
use snap7_rs::S7Client;
use tower_http::cors::{Any, CorsLayer};
use std::sync::{Arc, Mutex};

mod health_check;
mod read_from_plc;
mod write_to_plc;

use health_check::health_check_routes;
use read_from_plc::read_from_plc_routes;
use write_to_plc::write_to_plc_routes;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Mutex<Option<S7Client>>>,
}

pub fn create_routes(app_state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    Router::new()
        .merge(health_check_routes(app_state.clone()))
        .merge(read_from_plc_routes(app_state.clone()))
        .merge(write_to_plc_routes(app_state.clone()))
        .layer(cors)
}

pub async fn run() {
    let client = Arc::new(Mutex::new(None));

    // Initialize S7Client
    {
        let mut client_ref = client.lock().unwrap();
        *client_ref = Some(S7Client::create()); // Check if this is blocking or throws errors
    }

    let app_state = AppState {
        client: client.clone(),
    };

    let app = create_routes(app_state);

    // Use `tokio::net::TcpListener::bind` with proper error handling
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
