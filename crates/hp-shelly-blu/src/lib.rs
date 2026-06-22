use btleplug::{
    api::{Central, CentralEvent, Manager as _, ScanFilter, bleuuid::uuid_from_u16},
    platform::{Adapter, Manager},
};
use futures::stream::StreamExt;
use std::error::Error;
use uuid::Uuid;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

const BTHOME_ID: Uuid = uuid_from_u16(0xFCD2);

fn decode_bthome(payload: &[u8]) {
    if payload.is_empty() {
        return;
    }

    // If you are using a library like 'btsensor':
    // let result = btsensor::bthome::parse_v2(payload);

    // If doing it manually (e.g., viewing the raw array):
    let device_info_byte = payload[0];
    println!("Device Info Byte: {:#04X}", device_info_byte);
}

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().next().unwrap()
}

pub async fn log_devices() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await.unwrap();
    let central = get_central(&manager).await;
    let central_state = central.adapter_state().await.unwrap();
    println!("CentralState: {:?}", central_state);

    let mut events = central.events().await?;

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;
    while let Some(event) = events.next().await {
        if let CentralEvent::ServiceDataAdvertisement { id, service_data } = event
            && let Some(payload) = service_data.get(&BTHOME_ID)
        {
            println!("--- Received BTHome Packet ---");
            println!("Peripheral ID: {:?}", id);
            println!("Raw Payload (Hex): {:02X?}", payload);

            decode_bthome(payload);
        }
    }

    Ok(())
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
