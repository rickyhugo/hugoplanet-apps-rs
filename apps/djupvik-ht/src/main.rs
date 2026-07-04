use hp_shelly_blu::log_devices;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let _ = log_devices().await;
}
