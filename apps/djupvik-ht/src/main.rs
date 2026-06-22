use hp_shelly_blu::log_devices;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let _ = log_devices().await;
}
