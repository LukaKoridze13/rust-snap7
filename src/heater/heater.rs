use crate::routes::AppState;
use pid::Pid;
use snap7_rs::{AreaTable, WordLenTable};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
pub struct Heater {
    pub target_temp:f32,
    pub enabled: Arc<Mutex<bool>>, // Shared state for enabled
}

impl Heater {
    pub async fn enable(&self, app_state: &Arc<Mutex<AppState>>, pid: &Arc<Mutex<Pid<f64>>>) {
        let mut enabled = self.enabled.lock().await; // Lock the mutex to set the state
        *enabled = true;
        // Start the background task
        let heater_clone = self.clone();
        let heater_clone_for_task = Arc::clone(&heater_clone.enabled);
        let app_state_clone = Arc::clone(&app_state);
        let pid_clone = Arc::clone(&pid);
        tokio::spawn(async move {
            heater_clone
                .start_interval(heater_clone_for_task, app_state_clone, pid_clone)
                .await;
        });
    }

    pub async fn disable(&self) {
        let mut enabled = self.enabled.lock().await; // Lock the mutex to set the state
        *enabled = false;
    }

    fn water_present(&self, state: &AppState) -> bool {
        let mut buffer: [u8; 1] = [0; 1];
        state
            .s7_client
            .read_area(
                AreaTable::S7AreaPE,
                0,
                0,
                1,
                WordLenTable::S7WLBit,
                &mut buffer,
            )
            .unwrap();
        buffer[0] == 1
    }

    pub fn get_temperature(&self, state: &AppState) -> f32 {
        let mut buffer: [u8; 2] = [0; 2];
        state.s7_client.db_read(1, 0, 2, &mut buffer).unwrap();

        let raw_value = ((buffer[0] as u16) << 8) | (buffer[1] as u16);
        self.scale_to_temperature(raw_value)
    }

    fn scale_to_temperature(&self, raw_value: u16) -> f32 {
        let min_raw_value = 0;
        let max_raw_value = 27648;
        let min_temp = -40.0;
        let max_temp = 100.0;

        min_temp
            + (raw_value as f32 - min_raw_value as f32) * (max_temp - min_temp)
                / (max_raw_value as f32 - min_raw_value as f32)
    }

    async fn start_interval(
        &self,
        enabled: Arc<Mutex<bool>>,
        app_state: Arc<Mutex<AppState>>,
        pid: Arc<Mutex<Pid<f64>>>,
    ) {
        loop {
            {
                let enabled = enabled.lock().await;
                if !*enabled {
                    break;
                }
            }

            let app_state = app_state.lock().await;
            if !self.water_present(&*app_state) {
                // If water is not present, skip this iteration
                drop(app_state);
                sleep(Duration::from_millis(100)).await;
                continue;
            }

            let current_temperature = self.get_temperature(&*app_state);
            println!("Target Temp.: {:.2}", self.target_temp);

            println!("Temp.: {:.2}", current_temperature);

            let mut pid = pid.lock().await;
            let output: pid::ControlOutput<f64> =
                pid.next_control_output(current_temperature as f64);
            println!("Power %: {:.2}", output.output);
            let clamped_output = output.output.max(0.0).min(100.0);
            let on_duration = (clamped_output / 100.0 * 10000.0) as u64;
            let off_duration = 10000 - on_duration;
            drop(pid);

            // Turn on the heater
            {
                let mut byte_buff = [0u8; 1];
                let _read_result = app_state.s7_client.read_area(
                    snap7_rs::AreaTable::S7AreaPA,
                    0,
                    0,
                    1, // Reading 1 byte
                    WordLenTable::S7WLByte,
                    &mut byte_buff,
                );

                byte_buff[0] |= 1 << 1;
                let _write_result = app_state.s7_client.write_area(
                    snap7_rs::AreaTable::S7AreaPA, // Adjust if needed
                    0,                             // DB number - adjust if needed
                    0,
                    1, // Writing 1 byte
                    WordLenTable::S7WLByte,
                    &mut byte_buff, // Write the modified byte
                );
            }

            // Sleep for the on_duration
            sleep(Duration::from_millis(on_duration)).await;

            // Turn off the heater
            {
                let mut byte_buff = [0u8; 1];
                let _read_result = app_state.s7_client.read_area(
                    snap7_rs::AreaTable::S7AreaPA,
                    0,
                    0,
                    1, // Reading 1 byte
                    WordLenTable::S7WLByte,
                    &mut byte_buff,
                );

                byte_buff[0] &= !(1 << 1);
                let _write_result = app_state.s7_client.write_area(
                    snap7_rs::AreaTable::S7AreaPA, // Adjust if needed
                    0,                             // DB number - adjust if needed
                    0,
                    1, // Writing 1 byte
                    WordLenTable::S7WLByte,
                    &mut byte_buff, // Write the modified byte
                );
            }

            // Sleep for the off_duration
            sleep(Duration::from_millis(off_duration)).await;
        }
    }
}
