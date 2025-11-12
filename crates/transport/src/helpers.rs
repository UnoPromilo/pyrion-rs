use core::array::TryFromSliceError;

pub fn decode_f32(data: &[u8]) -> Result<f32, TryFromSliceError> {
    data.try_into()
        .map(f32::from_le_bytes)
}

pub fn decode_u64(data: &[u8]) -> Result<u64, TryFromSliceError> {
    data.try_into()
        .map(u64::from_le_bytes)
}

pub fn decode_u32(data: &[u8]) -> Result<u32, TryFromSliceError> {
    data.try_into()
        .map(u32::from_le_bytes)
}
