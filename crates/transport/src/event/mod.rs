use crate::packet::Packet;

pub mod decoder;
pub mod encoder;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Event {
    DeviceIntroduction(DeviceIntroduction), // 0x01
    Telemetry(Telemetry),                   // 0x02
}

impl Packet for Event {
    type Error = Error;

    fn deserialize(data: &[u8]) -> Result<Self, Self::Error> {
        let event_type = data[0];
        match event_type {
            0x01 => {
                let device_introduction = DeviceIntroduction::deserialize(&data[1..])?;
                Ok(Event::DeviceIntroduction(device_introduction))
            }
            0x02 => {
                let telemetry = Telemetry::deserialize(&data[1..])?;
                Ok(Event::Telemetry(telemetry))
            }
            _ => Err(Error::EventNotFound),
        }
    }

    fn serialize(&self, buffer: &mut [u8]) -> usize {
        match self {
            Event::DeviceIntroduction(device_introduction) => {
                buffer[0] = 0x01;
                let content_len = device_introduction.serialize(&mut buffer[1..]);
                1 + content_len
            }
            Event::Telemetry(telemetry) => {
                buffer[0] = 0x02;
                let content_len = telemetry.serialize(&mut buffer[1..]);
                1 + content_len
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Telemetry {
    pub cpu_temperature: f32,     // in kelvins
    pub motor_temperature: f32,   // in kelvins
    pub v_bus: f32,               // in volts
    pub power_consumption: f32,   // in watts
    pub current_consumption: f32, // in amperes
    pub duty_cycle: f32,          // 0-1
    pub uptime: u64,              // milliseconds
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DeviceIntroduction {
    pub uid: [u8; 12],
    pub firmware_version: [u8; 3],
}

impl Telemetry {
    pub fn serialize(&self, buffer: &mut [u8]) -> usize {
        buffer[..4].copy_from_slice(&self.cpu_temperature.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.motor_temperature.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.v_bus.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.power_consumption.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.current_consumption.to_le_bytes());
        buffer[20..24].copy_from_slice(&self.duty_cycle.to_le_bytes());
        buffer[24..32].copy_from_slice(&self.uptime.to_le_bytes());
        32
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Error> {
        let cpu_temperature = decode_f32(&data[0..4])?;
        let motor_temperature = decode_f32(&data[4..8])?;
        let v_bus = decode_f32(&data[8..12])?;
        let power_consumption = decode_f32(&data[12..16])?;
        let current_consumption = decode_f32(&data[16..20])?;
        let duty_cycle = decode_f32(&data[20..24])?;
        let uptime = decode_u64(&data[24..32])?;
        Ok(Self {
            cpu_temperature,
            motor_temperature,
            v_bus,
            power_consumption,
            current_consumption,
            duty_cycle,
            uptime,
        })
    }
}

impl DeviceIntroduction {
    pub fn serialize(&self, buffer: &mut [u8]) -> usize {
        buffer[..12].copy_from_slice(&self.uid);
        buffer[12..15].copy_from_slice(&self.firmware_version);
        15
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            uid: data[0..12].try_into().map_err(|_| Error::InvalidContent)?,
            firmware_version: data[12..15].try_into().map_err(|_| Error::InvalidContent)?,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    EventNotFound,
    InvalidContent,
}

fn decode_f32(data: &[u8]) -> Result<f32, Error> {
    data.try_into()
        .map(f32::from_le_bytes)
        .map_err(|_| Error::InvalidContent)
}

fn decode_u64(data: &[u8]) -> Result<u64, Error> {
    data.try_into()
        .map(u64::from_le_bytes)
        .map_err(|_| Error::InvalidContent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn telemetry_should_serialize_and_deserialize() {
        let telemetry = Telemetry {
            cpu_temperature: 23.0,
            motor_temperature: 33.0,
            v_bus: 12.3,
            power_consumption: 230.0,
            current_consumption: 27.56,
            duty_cycle: 22.2222,
            uptime: u64::MAX,
        };
        let mut buffer = [0u8; 256];
        let length = telemetry.serialize(&mut buffer);
        assert_eq!(length, 32);
        let deserialized = Telemetry::deserialize(&buffer[..length]).unwrap();
        assert_eq!(deserialized, telemetry);
    }

    #[test]
    pub fn device_introduction_should_serialize_and_deserialize() {
        let device_introduction = DeviceIntroduction {
            uid: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            firmware_version: [13, 14, 15],
        };
        let mut buffer = [0u8; 256];
        let length = device_introduction.serialize(&mut buffer);
        assert_eq!(length, 15);
        let deserialized = DeviceIntroduction::deserialize(&buffer[..length]).unwrap();
        assert_eq!(deserialized, device_introduction);
    }
}
