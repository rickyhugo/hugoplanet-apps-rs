pub mod bthome;

use btleplug::{
    api::{
        Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, bleuuid::uuid_from_u16,
    },
    platform::{Adapter, Manager},
};
use futures::stream::StreamExt;
use std::error::Error;
use uuid::Uuid;

use crate::bthome::Measurement;
use crate::bthome::decode;

const BTHOME_ID: Uuid = uuid_from_u16(0xFCD2);

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().next().unwrap()
}

pub async fn log_devices() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await.unwrap();
    let central = get_central(&manager).await;
    println!("CentralState: {:?}", central.adapter_state().await.unwrap());

    let mut events = central.events().await?;
    central.start_scan(ScanFilter::default()).await?;

    while let Some(event) = events.next().await {
        if let CentralEvent::ServiceDataAdvertisement { id, service_data } = event
            && let Some(payload) = service_data.get(&BTHOME_ID)
        {
            let mac = match central.peripheral(&id).await {
                Ok(p) => p.address().to_string(),
                Err(_) => id.to_string(),
            };
            println!("mac: {mac} hex: {:02X?}", payload);

            if let Ok(packet) = decode(payload) {
                for m in packet.measurements {
                    match m {
                        Measurement::Temperature(v) | Measurement::TemperatureSmall(v) => {
                            println!("  temperature: {v}")
                        }
                        Measurement::Humidity(v) => println!("  humidity: {v}"),
                        Measurement::HumidityShort(v) => println!("  humidity: {v}"),
                        Measurement::Battery(v) => println!("  battery: {v}%"),
                        Measurement::BatteryVoltage(v) => println!("  battery_voltage: {v}V"),
                        _ => println!("  {:?}", m),
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
