use snap7_rs::S7Client;
use std::thread;
use std::time::Duration;

fn main() {
    let client = S7Client::create();
    
    loop {
        match client.connect_to("192.168.0.1", 0, 2) {
            Ok(_) => {
                println!("Connected to PLC");
                break;
            }
            Err(e) => {
                println!("Error connecting to PLC: {:?}. Retrying in 1 second...", e);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
