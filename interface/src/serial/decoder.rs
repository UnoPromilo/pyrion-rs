use crate::command::Command::*;
use crate::command::{Command, Pid, PidValue};
use crate::serial::ArgList;
use crate::serial::errors::DecodingError;
use crate::serial::errors::DecodingError::*;
use crate::serial::parser::CommandEnvelope;
use core::str::FromStr;
use foc::state::ControlCommand;

pub fn decode_command(command_envelope: &CommandEnvelope) -> Result<Command, DecodingError> {
    match command_envelope.name {
        "echo" => Ok(Echo),
        "get" => decode_get_command(&command_envelope.args),
        "set_pid" => decode_set_pid(&command_envelope.args),
        "calibrate" => decode_calibrate_command(&command_envelope.args),
        "set_target" => decode_set_control_command(&command_envelope.args),
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
                "zero" => Ok(ControlCommand::SetTargetZero),
                "voltage" => Ok(ControlCommand::SetTargetVoltage(try_parse(value)?)),
                "torque" => Ok(ControlCommand::SetTargetTorque(try_parse(value)?)),
                "Velocity" => Ok(ControlCommand::SetTargetVelocity(try_parse(value)?)),
                "angle" => Ok(ControlCommand::SetTargetPosition(try_parse(value)?)),
                &_ => Err(InvalidArgumentValue),
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
fn decode_calibrate_command(args: &ArgList) -> Result<Command, DecodingError> {
    match args.items.len() {
        0 => Err(NotEnoughArguments),
        1 => {
            // Safe because we checked the length above.
            let argument = args.items.first().unwrap();
            let parameter = match argument.name {
                "shaft" => Ok(ControlCommand::CalibrateShaft),
                &_ => Err(InvalidArgumentValue),
            }?;
            Ok(SetControlCommand(parameter))
        }
        _ => Err(TooManyArguments),
    }
}

fn decode_get_command(args: &ArgList) -> Result<Command, DecodingError> {
    match args.items.len() {
        0 => Err(NotEnoughArguments),
        1 => {
            // Safe because we checked the length above.
            let argument = args.items.first().unwrap();
            match argument.name {
                "state" => Ok(GetState),
                "shaft" => Ok(GetShaft),
                &_ => Err(InvalidArgumentValue),
            }
        }
        _ => Err(TooManyArguments),
    }
}

fn decode_set_pid(args: &ArgList) -> Result<Command, DecodingError> {
    let pid = match args.items.first() {
        None => return Err(NotEnoughArguments),
        Some(pid) => match pid.name {
            "speed" => Pid::Velocity,
            "Current" => Pid::Current,
            _ => return Err(InvalidArgumentValue),
        },
    };
    let mut values = PidValue::default();
    for arg in args.items.iter().skip(1) {
        match arg.name {
            "kp" | "p" => {
                values.kp = try_parse(arg.value)?;
            }
            "ki" | "i" => {
                values.ki = try_parse(arg.value)?;
            }
            "kd" | "d" => {
                values.kd = try_parse(arg.value)?;
            }

            "kp_limit" | "p_limit" => {
                values.kp_limit.replace(try_parse(arg.value)?);
            }
            "ki_limit" | "i_limit" => {
                values.ki_limit.replace(try_parse(arg.value)?);
            }
            "kd_limit" | "d_limit" => {
                values.kd_limit.replace(try_parse(arg.value)?);
            }
            _ => return Err(InvalidArgumentValue),
        }
    }
    Ok(SetPid(pid, values))
}
