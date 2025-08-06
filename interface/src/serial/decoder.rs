use crate::command::Command;
use crate::command::Command::*;
use crate::serial::ArgList;
use crate::serial::errors::DecodingError;
use crate::serial::errors::DecodingError::*;
use crate::serial::parser::CommandEnvelope;
use core::str::FromStr;
use foc::state::ControlCommand;

pub fn decode_command(command_envelope: &CommandEnvelope) -> Result<Command, DecodingError> {
    match command_envelope.name {
        "echo" => Ok(Echo),
        "get_state" => Ok(GetState),
        "set_control_command" => decode_set_control_command(&command_envelope.args),
        &_ => Err(UnknownCommand),
    }
}

fn decode_set_control_command(args: &ArgList) -> Result<Command, DecodingError> {
    match args.items.len() {
        0 => Err(NotEnoughArguments),
        1 => {
            // Safe because we checked the length above.
            let argument = args.items.first().unwrap();
            let value = argument.value;
            let parameter = match argument.name {
                "none" => Ok(ControlCommand::None),
                "voltage" => Ok(ControlCommand::Voltage(try_parse(value)?)),
                "torque" => Ok(ControlCommand::Torque(try_parse(value)?)),
                "velocity" => Ok(ControlCommand::Velocity(try_parse(value)?)),
                "angle" => Ok(ControlCommand::Position(try_parse(value)?)),
                &_ => Err(InvalidArgumentValue)
            }?;
            Ok(SetControlCommand(parameter))
        }
        _ => Err(TooManyArguments),
    }
}

fn try_parse<T: FromStr>(value: Option<&str>) -> Result<T, DecodingError> {
    match value {
        None => Err(InvalidArgumentValue),
        Some(value) => match value.parse::<T>() {
            Err(_) => Err(InvalidArgumentValue),
            Ok(value) => Ok(value),
        },
    }
}
