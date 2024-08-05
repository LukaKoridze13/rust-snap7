use crate::{
    controllers::{self, SharedState},
    heater::Heater,
    middlewares::require_plc_connection,
};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use pid::Pid;
use serde::Deserialize;
use snap7_rs::S7Client;
use std::sync::Arc;
use std::{fmt, time::Duration};
use tokio::{sync::Mutex, time::sleep};

#[derive(Deserialize, Debug)]
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
        self.s7_client
            .connect_to(&self.address, self.rack, self.slot)
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

pub async fn create_routes() -> Router {
    let s7_client = Arc::new(S7Client::create());

    let app_state = Arc::new(Mutex::new(AppState {
        address: "192.168.0.1".to_string(),
        rack: 0,
        slot: 2,
        s7_client,
    }));

    {
        let app_state = app_state.clone();
        let app_state = app_state.lock().await;
        let connection_result = app_state.connect_to_plc();

        match connection_result {
            Ok(_) => println!(
                "** Connected to PLC. IP: {}, Rack: {}, Slot: {}",
                app_state.address, app_state.rack, app_state.slot
            ),
            Err(e) => println!("** Error connecting to PLC: {:?}", e),
        };
    }

    const TARGET_TEMP: f32 = 30.0;

    let heater = Heater {
        target_temp: TARGET_TEMP,
        enabled: Arc::new(tokio::sync::Mutex::new(true)),
    };

    let pid = Arc::new(tokio::sync::Mutex::new(Pid::new(TARGET_TEMP, 100.0)));

    let app_state_clone = app_state.clone();

    let shared_state = Arc::new(Mutex::new(SharedState {
        app_state,
        heater,
        pid,
    }));

    {
        let shared_state_guard = shared_state.lock().await;
        let mut pid_clone = shared_state_guard.pid.lock().await;

        // Set proportional, integral, and derivative gains
        pid_clone.p(1.0, 100.0); // Proportional gain with limit
        pid_clone.i(0.1, 100.0); // Integral gain with limit
        pid_clone.d(0.01, 100.0); // Derivative gain with limit
    }

    tokio::spawn(perform_periodic_task(
        app_state_clone,
        shared_state.clone(),
        TARGET_TEMP,
    ));

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");

    let health_check_router = Router::new()
        .route("/server", get(controllers::server_health_check))
        .route("/plc", get(controllers::plc_connection_check));

    let plc_router = Router::new()
        .route("/", get(controllers::get_plc_operating_mode))
        .route(
            "/configure_connection",
            post(controllers::change_plc_connection_settings),
        )
        .route("/stop", get(controllers::stop_plc))
        .route("/hot_start", get(controllers::hot_start))
        .route("/cold_start", get(controllers::cold_start))
        .layer(middleware::from_fn(require_plc_connection));

    let heater_router = Router::new()
        .route("/enable", get(controllers::enable_heater))
        .route("/disable", get(controllers::disable_heater))
        .with_state(shared_state.clone());

    Router::new()
        .nest("/health_check", health_check_router)
        .nest("/plc", plc_router)
        .nest("/heater", heater_router)
        .with_state(shared_state)
}

async fn perform_periodic_task(
    app_state_clone: Arc<Mutex<AppState>>,
    shared_state: Arc<Mutex<SharedState>>,
    target_temp: f32,
) {
    let mut update_interval = tokio::time::interval(Duration::from_millis(100));
    let mut heater_interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            _ = update_interval.tick() => {
                // Update values
                let app_state = app_state_clone.lock().await;
                let s7_client = &app_state.s7_client;

                let shared_state_guard = shared_state.lock().await;
                let mut pid_clone = shared_state_guard.pid.lock().await;

                let mut read_result = read_plc_db(s7_client).await;
                let temp = scale_to_temperature(read_result.temp_ai);
                read_result.current_temp = temp;
                read_result.target_temp = target_temp;
                read_result.heater_enabled = true;

                let output: pid::ControlOutput<f64> =
                    pid_clone.next_control_output(temp as f64);
                let power_percentage = output.output as f32;
                read_result.power_percentage = if power_percentage < 0.0 {
                    read_result.heater_enabled = false;
                    0.0
                } else if power_percentage > 100.0 {
                    100.0
                } else {
                    power_percentage
                };

                println!("Update task executed");

                write_plc_db(s7_client, &read_result).await;
            }
            _ = heater_interval.tick() => {
                // Control heater
                let app_state = app_state_clone.lock().await;
                let s7_client = &app_state.s7_client;

                let mut read_result = read_plc_db(s7_client).await;

                let total_duration = Duration::from_secs(10);
                let on_duration = (read_result.power_percentage / 100.0
                    * total_duration.as_millis() as f32)
                    as u64;
                let off_duration = total_duration.as_millis() as u64 - on_duration;

                // Set heater on for calculated duration
                read_result.heater_on = true;
                write_plc_db(s7_client, &read_result).await;

                // Sleep for the on_duration
                sleep(Duration::from_millis(on_duration)).await;

                // Set heater off
                read_result.heater_on = false;
                write_plc_db(s7_client, &read_result).await;

                // Sleep for the off_duration
                sleep(Duration::from_millis(off_duration)).await;
            }
        }
    }
}

#[derive(Debug)]
struct PlcData {
    temp_ai: u16,
    power_percentage: f32,
    target_temp: f32,
    current_temp: f32,
    water_present: bool,
    heater_enabled: bool,
    heater_on: bool,
}
impl PlcData {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.temp_ai.to_be_bytes());
        bytes.extend_from_slice(&self.power_percentage.to_be_bytes());
        bytes.extend_from_slice(&self.target_temp.to_be_bytes());
        bytes.extend_from_slice(&self.current_temp.to_be_bytes());

        // Combine water_present, heater_enabled, and heater_on into one byte
        let mut status_byte = 0u8;
        status_byte |= (self.water_present as u8) << 0;
        status_byte |= (self.heater_enabled as u8) << 1;
        status_byte |= (self.heater_on as u8) << 2;
        bytes.push(status_byte);

        bytes
    }
}

async fn read_plc_db(s7_client: &Arc<S7Client>) -> PlcData {
    let mut temp_ai = [0u8; 2];
    let mut power_percentage = [0u8; 4];
    let mut target_temp = [0u8; 4];
    let mut current_temp = [0u8; 4];
    let mut water_present = [0u8; 1];
    let mut heater_enabled = [0u8; 1];
    let mut heater_on = [0u8; 1];

    // Reading Temperature AI
    let temp_ai_value = if s7_client.db_read(1, 0, 2, &mut temp_ai).is_ok() {
        u16::from_be_bytes(temp_ai)
    } else {
        0
    };

    // Reading Power Percentage
    let power_percentage_value = if s7_client.db_read(1, 4, 4, &mut power_percentage).is_ok() {
        f32::from_bits(u32::from_be_bytes(power_percentage))
    } else {
        0.0
    };

    // Reading Target Temperature
    let target_temp_value = if s7_client.db_read(1, 8, 4, &mut target_temp).is_ok() {
        f32::from_bits(u32::from_be_bytes(target_temp))
    } else {
        0.0
    };

    // Reading Current Temperature
    let current_temp_value = if s7_client.db_read(1, 12, 4, &mut current_temp).is_ok() {
        f32::from_bits(u32::from_be_bytes(current_temp))
    } else {
        0.0
    };

    // Reading Water Present
    let water_present_value = if s7_client.db_read(1, 2, 1, &mut water_present).is_ok() {
        (water_present[0] >> 0 & 1) == 1
    } else {
        false
    };

    // Reading Heater Enabled
    let heater_enabled_value = if s7_client.db_read(1, 2, 1, &mut heater_enabled).is_ok() {
        (heater_enabled[0] >> 1 & 1) == 1
    } else {
        false
    };

    // Reading Heater On
    let heater_on_value = if s7_client.db_read(1, 2, 1, &mut heater_on).is_ok() {
        (heater_on[0] >> 2 & 1) == 1
    } else {
        false
    };

    PlcData {
        temp_ai: temp_ai_value,
        power_percentage: power_percentage_value,
        target_temp: target_temp_value,
        current_temp: current_temp_value,
        water_present: water_present_value,
        heater_enabled: heater_enabled_value,
        heater_on: heater_on_value,
    }
}

fn scale_to_temperature(raw_value: u16) -> f32 {
    let min_raw_value = 0;
    let max_raw_value = 27648;
    let min_temp = -40.0;
    let max_temp = 100.0;

    min_temp
        + (raw_value as f32 - min_raw_value as f32) * (max_temp - min_temp)
            / (max_raw_value as f32 - min_raw_value as f32)
}

async fn write_plc_db(s7_client: &Arc<S7Client>, data: &PlcData) {
    let mut data_bytes = data.to_bytes();

    // Write temperature AI (first 2 bytes)
    if let Err(err) = s7_client.db_write(1, 0, 2, &mut data_bytes[0..2]) {
        eprintln!("Error writing temp_ai: {:?}", err);
    }

    // Write power percentage (next 4 bytes)
    if let Err(err) = s7_client.db_write(1, 4, 4, &mut data_bytes[2..6]) {
        eprintln!("Error writing power_percentage: {:?}", err);
    }

    // Write target temperature (next 4 bytes)
    if let Err(err) = s7_client.db_write(1, 8, 4, &mut data_bytes[6..10]) {
        eprintln!("Error writing target_temp: {:?}", err);
    }

    // Write current temperature (next 4 bytes)
    if let Err(err) = s7_client.db_write(1, 12, 4, &mut data_bytes[10..14]) {
        eprintln!("Error writing current_temp: {:?}", err);
    }

    // Write status byte (next 1 byte)
    if let Err(err) = s7_client.db_write(1, 2, 1, &mut data_bytes[14..15]) {
        eprintln!("Error writing status byte: {:?}", err);
    }
}
