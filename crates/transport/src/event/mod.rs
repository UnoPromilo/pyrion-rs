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
    pub cpu_temperature: f32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DeviceIntroduction {
    pub uid: [u8; 12],
    pub firmware_version: [u8; 3],
}

impl Telemetry {
    pub fn serialize(&self, buffer: &mut [u8]) -> usize {
        let bytes = self.cpu_temperature.to_le_bytes();
        buffer[..4].copy_from_slice(&bytes);
        4
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Error> {
        let cpu_temperature = data
            .get(0..4)
            .ok_or(Error::InvalidContent)?
            .try_into()
            .map(f32::from_le_bytes)
            .map_err(|_| Error::InvalidContent)?;

        Ok(Self { cpu_temperature })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn telemetry_should_serialize_and_deserialize() {
        let telemetry = Telemetry {
            cpu_temperature: 23.0,
        };
        let mut buffer = [0u8; 256];
        let length = telemetry.serialize(&mut buffer);
        assert_eq!(length, 4);
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
