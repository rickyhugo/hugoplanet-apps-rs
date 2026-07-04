use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DecodeError {
    #[error("empty payload")]
    Empty,
    #[error("invalid version byte: 0x{0:02x}")]
    InvalidVersion(u8),
    #[error("encrypted payloads are not supported")]
    Encrypted,
    #[error("unexpected end of data")]
    Truncated,
    #[error("unknown object ID: 0x{0:02x}")]
    UnknownObjectId(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Measurement {
    PacketId(u8),
    Battery(u8),
    Temperature(f64),
    TemperatureSmall(f64),
    Humidity(f64),
    HumidityShort(u8),
    Pressure(f64),
    Illuminance(f64),
    Mass(f64),
    DewPoint(f64),
    Count(u8),
    Count16(u16),
    Count32(u32),
    Energy(f64),
    Power(f64),
    VoltageSmall(f64),
    Voltage(f64),
    Current(f64),
    CarbonDioxide(u16),
    TotalVolatileOrganicCompounds(u16),
    Moisture(f64),
    MoistureShort(u8),
    BatteryLow(bool),
    BatteryCharging(bool),
    Gas(f64),
    Acceleration(f64),
    Pm25(u16),
    Pm10(u16),
    Bool(bool),
    Button { press_count: u8 },
    Dimmer { direction: u8, steps: u8 },
    Irradiance(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BthomeV2Packet {
    pub measurements: Vec<Measurement>,
}

pub fn decode(data: &[u8]) -> Result<BthomeV2Packet, DecodeError> {
    if data.is_empty() {
        return Err(DecodeError::Empty);
    }

    let device_info = data[0];
    if device_info & 0x01 != 0 {
        return Err(DecodeError::Encrypted);
    }
    if (device_info & 0xE0) >> 5 != 2 {
        return Err(DecodeError::InvalidVersion(device_info));
    }

    let mut measurements = Vec::new();
    let mut pos = 1;

    while pos + 2 <= data.len() {
        let object_id = data[pos];
        pos += 1;

        let (measurement, consumed) = parse_element(object_id, &data[pos..])?;
        pos += consumed;
        measurements.push(measurement);
    }

    Ok(BthomeV2Packet { measurements })
}

fn read_u8(data: &[u8]) -> Result<(u8, usize), DecodeError> {
    data.first()
        .copied()
        .map(|v| (v, 1))
        .ok_or(DecodeError::Truncated)
}

fn read_u16(data: &[u8]) -> Result<(u16, usize), DecodeError> {
    let bytes = data.get(0..2).ok_or(DecodeError::Truncated)?;
    Ok((u16::from_le_bytes([bytes[0], bytes[1]]), 2))
}

fn read_i16(data: &[u8]) -> Result<(i16, usize), DecodeError> {
    let bytes = data.get(0..2).ok_or(DecodeError::Truncated)?;
    Ok((i16::from_le_bytes([bytes[0], bytes[1]]), 2))
}

fn read_u24(data: &[u8]) -> Result<(u32, usize), DecodeError> {
    let bytes = data.get(0..3).ok_or(DecodeError::Truncated)?;
    Ok((
        u32::from(bytes[0]) | u32::from(bytes[1]) << 8 | u32::from(bytes[2]) << 16,
        3,
    ))
}

fn read_u32(data: &[u8]) -> Result<(u32, usize), DecodeError> {
    let bytes = data.get(0..4).ok_or(DecodeError::Truncated)?;
    Ok((
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        4,
    ))
}

fn parse_element(id: u8, data: &[u8]) -> Result<(Measurement, usize), DecodeError> {
    match id {
        0x00 => {
            let (v, n) = read_u8(data)?;
            Ok((Measurement::PacketId(v), n))
        }
        0x01 => {
            let (v, n) = read_u8(data)?;
            Ok((Measurement::Battery(v), n))
        }
        0x02 => {
            let (v, n) = read_i16(data)?;
            Ok((Measurement::TemperatureSmall(v as f64 / 100.0), n))
        }
        0x03 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Humidity(v as f64 / 100.0), n))
        }
        0x04 => {
            let (v, n) = read_u24(data)?;
            Ok((Measurement::Pressure(v as f64 / 100.0), n))
        }
        0x05 => {
            let (v, n) = read_u24(data)?;
            Ok((Measurement::Illuminance(v as f64 / 100.0), n))
        }
        0x06 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Mass(v as f64 / 100.0), n))
        }
        0x07 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Mass(v as f64 / 100.0), n))
        }
        0x08 => {
            let (v, n) = read_i16(data)?;
            Ok((Measurement::DewPoint(v as f64 / 100.0), n))
        }
        0x09 => {
            let (v, n) = read_u8(data)?;
            Ok((Measurement::Count(v), n))
        }
        0x0A => {
            let (v, n) = read_u24(data)?;
            Ok((Measurement::Energy(v as f64 / 1000.0), n))
        }
        0x0B => {
            let (v, n) = read_u24(data)?;
            Ok((Measurement::Power(v as f64 / 100.0), n))
        }
        0x0C => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::VoltageSmall(v as f64 / 1000.0), n))
        }
        0x0D => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Pm25(v), n))
        }
        0x0E => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Pm10(v), n))
        }
        0x12 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::CarbonDioxide(v), n))
        }
        0x13 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::TotalVolatileOrganicCompounds(v), n))
        }
        0x14 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Moisture(v as f64 / 100.0), n))
        }
        0x15 => {
            let (v, n) = read_u8(data)?;
            Ok((Measurement::BatteryLow(v != 0), n))
        }
        0x16 => {
            let (v, n) = read_u8(data)?;
            Ok((Measurement::BatteryCharging(v != 0), n))
        }

        0x0F..=0x11 | 0x17..=0x2D => {
            let (v, n) = read_u8(data)?;
            Ok((Measurement::Bool(v != 0), n))
        }
        0x2E => {
            let (v, n) = read_u8(data)?;
            Ok((Measurement::HumidityShort(v), n))
        }
        0x2F => {
            let (v, n) = read_u8(data)?;
            Ok((Measurement::MoistureShort(v), n))
        }
        0x3A => {
            let (v, n) = read_u8(data)?;
            Ok((Measurement::Button { press_count: v }, n))
        }
        0x3C => {
            if data.first() == Some(&0x00) {
                Ok((
                    Measurement::Dimmer {
                        direction: 0,
                        steps: 0,
                    },
                    1,
                ))
            } else {
                let bytes = data.get(0..2).ok_or(DecodeError::Truncated)?;
                Ok((
                    Measurement::Dimmer {
                        direction: bytes[0],
                        steps: bytes[1],
                    },
                    2,
                ))
            }
        }
        0x3D => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Count16(v), n))
        }
        0x3E => {
            let (v, n) = read_u32(data)?;
            Ok((Measurement::Count32(v), n))
        }
        0x3F => {
            let (v, n) = read_i16(data)?;
            Ok((Measurement::Mass(v as f64 / 10.0), n))
        }
        0x40 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Mass(v as f64), n))
        }
        0x41 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Mass(v as f64 / 10.0), n))
        }
        0x42 => {
            let (v, n) = read_u24(data)?;
            Ok((Measurement::Energy(v as f64 / 1000.0), n))
        }
        0x43 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Current(v as f64 / 1000.0), n))
        }
        0x44 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Mass(v as f64 / 100.0), n))
        }
        0x45 => {
            let (v, n) = read_i16(data)?;
            Ok((Measurement::Temperature(v as f64 / 10.0), n))
        }
        0x46 => {
            let (v, n) = read_u8(data)?;
            Ok((Measurement::Irradiance(v as f64 / 10.0), n))
        }
        0x47 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Mass(v as f64), n))
        }
        0x48 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Mass(v as f64), n))
        }
        0x49 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Mass(v as f64 / 1000.0), n))
        }
        0x4A => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Voltage(v as f64 / 10.0), n))
        }
        0x4B => {
            let (v, n) = read_u24(data)?;
            Ok((Measurement::Gas(v as f64 / 1000.0), n))
        }
        0x4C => {
            let (v, n) = read_u32(data)?;
            Ok((Measurement::Gas(v as f64 / 1000.0), n))
        }
        0x4D => {
            let (v, n) = read_u32(data)?;
            Ok((Measurement::Energy(v as f64 / 1000.0), n))
        }
        0x4E => {
            let (v, n) = read_u32(data)?;
            Ok((Measurement::Mass(v as f64 / 1000.0), n))
        }
        0x4F => {
            let (v, n) = read_u32(data)?;
            Ok((Measurement::Mass(v as f64 / 1000.0), n))
        }
        0x50 => {
            let (v, n) = read_u32(data)?;
            Ok((Measurement::Count32(v), n))
        }
        0x51 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Acceleration(v as f64 / 1000.0), n))
        }
        0x52 => {
            let (v, n) = read_u16(data)?;
            Ok((Measurement::Acceleration(v as f64 / 1000.0), n))
        }
        _ => Err(DecodeError::UnknownObjectId(id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_payload() {
        assert_eq!(decode(&[]), Err(DecodeError::Empty));
    }

    #[test]
    fn test_encrypted_payload() {
        assert_eq!(decode(&[0x41]), Err(DecodeError::Encrypted));
    }

    #[test]
    fn test_invalid_version() {
        assert_eq!(decode(&[0x10]), Err(DecodeError::InvalidVersion(0x10)));
    }

    #[test]
    fn test_shelly_payload() {
        let data = [0x44, 0x00, 0xE2, 0x01, 0x64, 0x2E, 0x44, 0x45, 0xCF, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements.len(), 4);
        assert_eq!(packet.measurements[0], Measurement::PacketId(0xE2));
        assert_eq!(packet.measurements[1], Measurement::Battery(100));
        assert_eq!(packet.measurements[2], Measurement::HumidityShort(0x44));
        assert_eq!(packet.measurements[3], Measurement::Temperature(20.7));
    }

    #[test]
    fn test_version_0x40() {
        let data = [0x40, 0x01, 0x64];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Battery(100)]);
    }

    #[test]
    fn test_version_0x44() {
        let data = [0x44, 0x01, 0x64];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Battery(100)]);
    }

    #[test]
    fn test_packet_id() {
        let data = [0x40, 0x00, 0x01];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::PacketId(1)]);
    }

    #[test]
    fn test_battery() {
        let data = [0x40, 0x01, 0x50];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Battery(80)]);
    }

    #[test]
    fn test_temperature_small() {
        let data = [0x40, 0x02, 0xC4, 0x09];
        let packet = decode(&data).unwrap();
        assert_eq!(
            packet.measurements,
            vec![Measurement::TemperatureSmall(25.00)]
        );
    }

    #[test]
    fn test_temperature() {
        let data = [0x40, 0x45, 0xCF, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Temperature(20.7)]);
    }

    #[test]
    fn test_temperature_negative_small() {
        let data = [0x40, 0x02, 0xFE, 0xFF];
        let packet = decode(&data).unwrap();
        assert_eq!(
            packet.measurements,
            vec![Measurement::TemperatureSmall(-0.02)]
        );
    }

    #[test]
    fn test_humidity() {
        let data = [0x40, 0x03, 0x10, 0x27];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Humidity(100.00)]);
    }

    #[test]
    fn test_humidity_short() {
        let data = [0x40, 0x2E, 0x44];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::HumidityShort(68)]);
    }

    #[test]
    fn test_pressure() {
        // 101300 → 1013.00 hPa
        let data = [0x40, 0x04, 0xB4, 0x8B, 0x01];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Pressure(1013.00)]);
    }

    #[test]
    fn test_illuminance() {
        // 100000 → 1000.00 lux
        let data = [0x40, 0x05, 0xA0, 0x86, 0x01];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Illuminance(1000.00)]);
    }

    #[test]
    fn test_mass() {
        let data = [0x40, 0x06, 0x10, 0x27];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Mass(100.00)]);
    }

    #[test]
    fn test_dew_point() {
        let data = [0x40, 0x08, 0x2C, 0x01];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::DewPoint(3.00)]);
    }

    #[test]
    fn test_count() {
        let data = [0x40, 0x09, 0x2A];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Count(42)]);
    }

    #[test]
    fn test_energy() {
        let data = [0x40, 0x0A, 0xE8, 0x03, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Energy(1.000)]);
    }

    #[test]
    fn test_power() {
        let data = [0x40, 0x0B, 0x10, 0x27, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Power(100.00)]);
    }

    #[test]
    fn test_voltage_small() {
        let data = [0x40, 0x0C, 0xB8, 0x0B];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::VoltageSmall(3.000)]);
    }

    #[test]
    fn test_voltage() {
        let data = [0x40, 0x4A, 0x14, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Voltage(2.0)]);
    }

    #[test]
    fn test_current() {
        let data = [0x40, 0x43, 0xE8, 0x03];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Current(1.000)]);
    }

    #[test]
    fn test_co2() {
        let data = [0x40, 0x12, 0xF4, 0x01];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::CarbonDioxide(500)]);
    }

    #[test]
    fn test_tvoc() {
        let data = [0x40, 0x13, 0x64, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(
            packet.measurements,
            vec![Measurement::TotalVolatileOrganicCompounds(100)]
        );
    }

    #[test]
    fn test_moisture() {
        let data = [0x40, 0x14, 0x10, 0x27];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Moisture(100.00)]);
    }

    #[test]
    fn test_moisture_short() {
        let data = [0x40, 0x2F, 0x2D];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::MoistureShort(45)]);
    }

    #[test]
    fn test_battery_low() {
        let data = [0x40, 0x15, 0x01];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::BatteryLow(true)]);
    }

    #[test]
    fn test_battery_charging() {
        let data = [0x40, 0x16, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(
            packet.measurements,
            vec![Measurement::BatteryCharging(false)]
        );
    }

    #[test]
    fn test_gas_u24() {
        let data = [0x40, 0x4B, 0xE8, 0x03, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Gas(1.000)]);
    }

    #[test]
    fn test_acceleration() {
        let data = [0x40, 0x51, 0xE8, 0x03];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Acceleration(1.000)]);
    }

    #[test]
    fn test_bool_variants() {
        let data = [0x40, 0x0F, 0x01, 0x10, 0x00, 0x11, 0x01];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements.len(), 3);
        assert_eq!(packet.measurements[0], Measurement::Bool(true));
        assert_eq!(packet.measurements[1], Measurement::Bool(false));
        assert_eq!(packet.measurements[2], Measurement::Bool(true));
    }

    #[test]
    fn test_unknown_object_id() {
        let data = [0x40, 0xFF, 0x00];
        assert_eq!(decode(&data), Err(DecodeError::UnknownObjectId(0xFF)));
    }

    #[test]
    fn test_truncated_element() {
        let data = [0x40, 0x02, 0x00];
        assert_eq!(decode(&data), Err(DecodeError::Truncated));
    }

    #[test]
    fn test_no_elements() {
        let data = [0x40];
        let packet = decode(&data).unwrap();
        assert!(packet.measurements.is_empty());
    }

    #[test]
    fn test_single_remaining_byte() {
        // Need at least 2 bytes for an element (id + 1 data byte)
        let data = [0x40, 0x00];
        let packet = decode(&data).unwrap();
        assert!(packet.measurements.is_empty());
    }

    #[test]
    fn test_pm25() {
        let data = [0x40, 0x0D, 0x19, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Pm25(25)]);
    }

    #[test]
    fn test_pm10() {
        let data = [0x40, 0x0E, 0x28, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Pm10(40)]);
    }

    #[test]
    fn test_button_event() {
        let data = [0x44, 0x3A, 0x00, 0x3A, 0x05];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements.len(), 2);
        assert_eq!(
            packet.measurements[0],
            Measurement::Button { press_count: 0 }
        );
        assert_eq!(
            packet.measurements[1],
            Measurement::Button { press_count: 5 }
        );
    }

    #[test]
    fn test_dimmer_event() {
        let data = [0x44, 0x3C, 0x00, 0x3C, 0x01, 0x03];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements.len(), 2);
        assert_eq!(
            packet.measurements[0],
            Measurement::Dimmer {
                direction: 0,
                steps: 0
            }
        );
        assert_eq!(
            packet.measurements[1],
            Measurement::Dimmer {
                direction: 1,
                steps: 3
            }
        );
    }

    #[test]
    fn test_multi_sensor_mho() {
        // From btsensor test: PacketId(168), Battery(100), TempSmall(2507), Humidity(4390)
        let data = [64, 0, 168, 1, 100, 2, 203, 9, 3, 38, 17];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements.len(), 4);
        assert_eq!(packet.measurements[0], Measurement::PacketId(168));
        assert_eq!(packet.measurements[1], Measurement::Battery(100));
        assert_eq!(packet.measurements[2], Measurement::TemperatureSmall(25.07));
        assert_eq!(packet.measurements[3], Measurement::Humidity(43.90));
    }

    #[test]
    fn test_irradiance() {
        let data = [0x40, 0x46, 0x2C];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Irradiance(4.4)]);
    }

    #[test]
    fn test_count16() {
        let data = [0x40, 0x3D, 0x2A, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Count16(42)]);
    }

    #[test]
    fn test_count32() {
        let data = [0x40, 0x3E, 0x2A, 0x00, 0x00, 0x00];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Count32(42)]);
    }

    #[test]
    fn test_door_open() {
        let data = [0x40, 0x1A, 0x01];
        let packet = decode(&data).unwrap();
        assert_eq!(packet.measurements, vec![Measurement::Bool(true)]);
    }
}
