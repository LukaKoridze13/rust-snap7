use snap7_rs::S7Client;

fn main() {
    let client = S7Client::create();
    if let Err(e) = client.connect_to("192.168.0.1", 0, 1) {
        println!("Error PLC: {:?}", e);
    } else {
        println!("Connected to PLC");
    }
}