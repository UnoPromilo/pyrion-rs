use core::fmt::Debug;

pub trait Packet: Sized + Debug {
    type Error;
    fn deserialize(data: &[u8]) -> Result<Self, Self::Error>;
    fn serialize(&self, buffer: &mut [u8]) -> usize;
}
