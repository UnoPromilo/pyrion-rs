#![no_std]

#[derive(PartialEq, Debug)]
pub enum Command {
    SetTargetSpeed(i16), // RPM
    GetInfo,
}

#[derive(PartialEq, Debug)]
pub enum ParsingError {
    EmptyCommand,
    UnknownCommand,
    MissingArgument,
    InvalidArgument,
}

pub fn parse_command(str: &str) -> Result<Command, ParsingError> {
    use Command::*;
    use ParsingError::*;
    let mut tokens = str.split_whitespace();
    let command = tokens.next();
    match command {
        None => Err(EmptyCommand),
        Some("set_speed") => {
            let value = tokens
                .next()
                .ok_or(MissingArgument)?
                .parse::<i16>()
                .map_err(|_| InvalidArgument)?;
            Ok(SetTargetSpeed(value))
        }
        Some("get_info") => Ok(GetInfo),
        _ => Err(UnknownCommand),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_command() {
        assert_eq!(parse_command(""), Err(ParsingError::EmptyCommand));
        assert_eq!(parse_command("   "), Err(ParsingError::EmptyCommand));
    }

    #[test]
    fn test_unknown_command() {
        assert_eq!(parse_command("fly"), Err(ParsingError::UnknownCommand));
        assert_eq!(
            parse_command("set_thrust 1000"),
            Err(ParsingError::UnknownCommand)
        );
    }

    #[test]
    fn test_get_info() {
        assert_eq!(parse_command("get_info"), Ok(Command::GetInfo));
        assert_eq!(parse_command("  get_info   "), Ok(Command::GetInfo));
    }

    #[test]
    fn test_set_speed_valid() {
        assert_eq!(
            parse_command("set_speed 1500"),
            Ok(Command::SetTargetSpeed(1500))
        );
        assert_eq!(
            parse_command(" set_speed -500 "),
            Ok(Command::SetTargetSpeed(-500))
        );
    }

    #[test]
    fn test_set_speed_missing_arg() {
        assert_eq!(
            parse_command("set_speed"),
            Err(ParsingError::MissingArgument)
        );
    }

    #[test]
    fn test_set_speed_invalid_arg() {
        assert_eq!(
            parse_command("set_speed abc"),
            Err(ParsingError::InvalidArgument)
        );
        assert_eq!(
            parse_command("set_speed 12.34"),
            Err(ParsingError::InvalidArgument)
        );
    }
}
