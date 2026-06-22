use hp_shelly_blu::log_devices;

#[tokio::main]
async fn main() {
    let _ = log_devices().await;
}
