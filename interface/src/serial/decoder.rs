use crate::command::Command;
use crate::serial::errors::DecodingError;
use crate::serial::parser::CommandEnvelope;

pub fn decode_command(command_envelope: &CommandEnvelope) -> Result<Command, DecodingError> {
    match command_envelope.name {
        "echo" => Ok(Command::Echo),
        "get_state" => Ok(Command::GetState),
        &_ => Err(DecodingError::UnknownCommand),
    }
}
