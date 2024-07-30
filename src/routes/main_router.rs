use std::sync::{Arc, Mutex};
use serde::Deserialize;
use axum::{middleware, routing::{get, post}, Router};
use snap7_rs::S7Client;
use std::fmt;
use crate::{controllers, middlewares::require_plc_connection};


#[derive(Deserialize,Debug)]
pub struct PLCConfig {
    pub address: String,
    pub rack: i32,
    pub slot: i32,
}

#[derive(Clone)]
pub struct AppState {
    pub s7_client: Arc<S7Client>,
    pub address: String,
    pub rack: i32,
    pub slot: i32,
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("address", &self.address)
            .field("rack", &self.rack)
            .field("slot", &self.slot)
            .finish()
    }
}


impl AppState {
    pub fn connect_to_plc(&self) -> Result<(), anyhow::Error> {
        self.s7_client.connect_to(&self.address, self.rack, self.slot)
    }

    pub fn get_plc_status(&self) -> Result<i32, anyhow::Error> {
        let mut status = 0;
        self.s7_client.get_plc_status(&mut status)?;
        Ok(status)
    }

    pub fn update_config(&mut self, new_config: PLCConfig) {
        self.address = new_config.address;
        self.rack = new_config.rack;
        self.slot = new_config.slot;
    }
}

pub fn create_routes() -> Router {
    let s7_client = Arc::new(S7Client::create());

    let app_state = Arc::new(Mutex::new(AppState {
        address: "192.168.0.1".to_string(),
        rack: 0,
        slot: 2,
        s7_client,
    }));

    {
        let app_state = app_state.clone();
        let app_state = app_state.lock().unwrap();
        let connection_result = app_state.connect_to_plc();

        match connection_result {
            Ok(_) => println!("** Connected to PLC. IP: {}, Rack: {}, Slot: {}", app_state.address, app_state.rack, app_state.slot),
            Err(e) => println!("** Error connecting to PLC: {:?}", e),
        };
    }

    let health_check_router = Router::new()
        .route("/server", get(controllers::server_health_check))
        .route("/plc", get(controllers::plc_connection_check));

    let plc_router = Router::new()
        .route("/", get(controllers::get_plc_operating_mode))
        .route("/configure_connection", post(controllers::change_plc_connection_settings))
        .route("/stop", get(controllers::stop_plc))
        .route("/hot_start", get(controllers::hot_start))
        .route("/cold_start", get(controllers::cold_start))

        .layer(middleware::from_fn(require_plc_connection));



    Router::new()
        .nest("/health_check", health_check_router)
        .nest("/plc", plc_router)
        .with_state(app_state)
}



