#[derive(Debug)]
pub enum DecoderError {
    IoError(std::io::Error),
    DecodeError(transport::decoder::DecoderError<transport::event::EventDeserializationError>),
}

impl From<std::io::Error> for DecoderError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<transport::decoder::DecoderError<transport::event::EventDeserializationError>> for DecoderError {
    fn from(e: transport::decoder::DecoderError<transport::event::EventDeserializationError>) -> Self {
        Self::DecodeError(e)
    }
}

#[derive(Debug)]
pub enum EncoderError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for EncoderError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}
