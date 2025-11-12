use crate::helpers::{decode_f32, decode_u32, decode_u64};
use crate::packet::Packet;
use core::array::TryFromSliceError;

pub mod decoder;
pub mod encoder;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Event {
    DeviceIntroduction(DeviceIntroduction), // 0x01
    Telemetry(Telemetry),                   // 0x02
    Success,                                // 0x03
    Failure,                                // 0x04
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
            0x03 => Ok(Event::Success),
            0x04 => Ok(Event::Failure),
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
            Event::Success => {
                buffer[0] = 0x03;
                1
            }
            Event::Failure => {
                buffer[0] = 0x04;
                1
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Telemetry {
    pub cpu_temperature: f32,     // in kelvins
    pub driver_temperature: f32,  // in kelvins
    pub motor_temperature: f32,   // in kelvins
    pub v_bus: f32,               // in volts
    pub power_consumption: f32,   // in watts
    pub current_consumption: f32, // in amperes
    pub duty_cycle: f32,          // 0-1
    pub uptime: u64,              // milliseconds
    pub ongoing_errors: u32,
    pub resolved_errors: u32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DeviceIntroduction {
    pub uid: [u8; 12],
    pub firmware_version: [u8; 3],
}

impl Telemetry {
    pub fn serialize(&self, buffer: &mut [u8]) -> usize {
        buffer[..4].copy_from_slice(&self.cpu_temperature.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.driver_temperature.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.motor_temperature.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.v_bus.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.power_consumption.to_le_bytes());
        buffer[20..24].copy_from_slice(&self.current_consumption.to_le_bytes());
        buffer[24..28].copy_from_slice(&self.duty_cycle.to_le_bytes());
        buffer[28..36].copy_from_slice(&self.uptime.to_le_bytes());
        buffer[36..40].copy_from_slice(&self.ongoing_errors.to_le_bytes());
        buffer[40..44].copy_from_slice(&self.resolved_errors.to_le_bytes());
        44
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Error> {
        let cpu_temperature = decode_f32(&data[0..4])?;
        let driver_temperature = decode_f32(&data[4..8])?;
        let motor_temperature = decode_f32(&data[8..12])?;
        let v_bus = decode_f32(&data[12..16])?;
        let power_consumption = decode_f32(&data[16..20])?;
        let current_consumption = decode_f32(&data[20..24])?;
        let duty_cycle = decode_f32(&data[24..28])?;
        let uptime = decode_u64(&data[28..36])?;
        let ongoing_errors = decode_u32(&data[36..40])?;
        let resolved_errors = decode_u32(&data[40..44])?;
        Ok(Self {
            cpu_temperature,
            driver_temperature,
            motor_temperature,
            v_bus,
            power_consumption,
            current_consumption,
            duty_cycle,
            uptime,
            ongoing_errors,
            resolved_errors,
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

impl From<TryFromSliceError> for Error {
    fn from(_: TryFromSliceError) -> Self {
        Error::InvalidContent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn telemetry_should_serialize_and_deserialize() {
        let telemetry = Telemetry {
            cpu_temperature: 23.0,
            driver_temperature: 55.0,
            motor_temperature: 33.0,
            v_bus: 12.3,
            power_consumption: 230.0,
            current_consumption: 27.56,
            duty_cycle: 22.2222,
            uptime: u64::MAX,
            ongoing_errors: 2,
            resolved_errors: 4,
        };
        let mut buffer = [0u8; 256];
        let length = telemetry.serialize(&mut buffer);
        assert_eq!(length, 44);
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

    #[test]
    pub fn success_event() {
        let mut buffer = [0; 100];
        let len = Event::Success.serialize(&mut buffer);
        let result = Event::deserialize(&buffer[..len]);
        assert!(result.is_ok());
    }

    #[test]
    pub fn failure_event() {
        let mut buffer = [0; 100];
        let len = Event::Failure.serialize(&mut buffer);
        let result = Event::deserialize(&buffer[..len]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Event::Failure);
    }
}
