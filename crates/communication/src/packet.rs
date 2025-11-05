pub const PACKET_SIZE: usize = 64;

#[derive(Debug, Copy, Clone)]
pub struct Packet {
    pub interface: Interface,
    pub buffer: [u8; PACKET_SIZE],
    pub length: usize,
}

#[derive(Debug, Copy, Clone)]
pub enum Interface {
    Serial,
    Usb,
}

impl Packet {
    pub fn empty(interface: Interface) -> Self {
        Self {
            interface,
            buffer: [0u8; PACKET_SIZE],
            length: 0,
        }
    }

    pub fn from_slice(slice: &[u8], interface: Interface) -> Self {
        assert!(slice.len() <= PACKET_SIZE, "Slice is too large");
        let mut buffer = [0u8; PACKET_SIZE];
        buffer[..slice.len()].copy_from_slice(slice);
        Self {
            interface,
            buffer,
            length: slice.len(),
        }
    }
}

pub fn split_into_packets<'a>(
    data: &'a [u8],
    interface: Interface,
) -> impl Iterator<Item = Packet> + 'a {
    core::iter::successors(Some(data), |overflow_data| {
        (overflow_data.len() > PACKET_SIZE).then(|| &overflow_data[PACKET_SIZE..])
    })
    .map(move |chunk| {
        let take = chunk.len().min(PACKET_SIZE);
        Packet::from_slice(&chunk[..take], interface)
    })
    .chain(
        (!data.is_empty() && data.len().is_multiple_of(PACKET_SIZE))
            .then(|| Packet::empty(interface)),
    )
}
