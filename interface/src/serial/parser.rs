use crate::serial::errors::ParsingError;
use core::slice::Iter;
use heapless::Vec;

pub struct CommandEnvelope<'a> {
    pub name: &'a str,
    pub args: ArgList<'a>,
}

pub struct ArgList<'a> {
    items: Vec<Arg<'a>, 8>,
}

impl ArgList<'_> {
    pub fn iter(&self) -> Iter<'_, Arg> {
        self.items.iter()
    }
}

pub struct Arg<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
}

pub struct RawCommand<'a> {
    value: &'a str,
}

impl<'a> RawCommand<'a> {
    pub fn serial(input: &'a str) -> Self {
        Self { value: input }
    }
}

pub fn parse<'a>(raw_command: &RawCommand<'a>) -> Result<CommandEnvelope<'a>, ParsingError> {
    let mut tokens = raw_command.value.trim().split_whitespace();
    let name = tokens.next().ok_or(ParsingError::Empty)?;
    if is_name_expression_valid(name) == false {
        return Err(ParsingError::InvalidCommandFormat);
    }

    let mut arguments = Vec::new();
    while let Some(token) = tokens.next() {
        let mut splitted = token.trim().split('=');
        let name = splitted
            .next()
            .ok_or(ParsingError::InvalidArgumentFormat)?
            .trim();
        if is_name_expression_valid(name) == false {
            return Err(ParsingError::InvalidArgumentFormat);
        }
        let value: Option<&str> = match splitted.next() {
            Some(value) if is_value_expression_valid(value) => Some(value),
            Some(_) => return Err(ParsingError::InvalidArgumentFormat),
            None => None,
        };
        if splitted.next().is_some() {
            return Err(ParsingError::InvalidArgumentFormat);
        }

        arguments
            .push(Arg { name, value })
            .map_err(|_| ParsingError::TooManyArguments)?;
    }

    Ok(CommandEnvelope {
        name,
        args: ArgList { items: arguments },
    })
}

fn is_name_expression_valid(raw_name: &str) -> bool {
    raw_name.chars().enumerate().all(|(i, c)| {
        if i == 0 {
            c.is_ascii_alphabetic()
        } else {
            c.is_ascii_alphanumeric() || c == '_'
        }
    })
}

fn is_value_expression_valid(raw_name: &str) -> bool {
    raw_name.chars().enumerate().all(|(i, c)| {
        if i == 0 {
            c.is_ascii_alphanumeric()
        } else {
            c.is_ascii_alphanumeric() || c == '_'
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::serial::parser::{RawCommand, parse};
    use alloc::{format, vec};

    #[test]
    fn parse_empty_command_name_should_fail() {
        for input in ["", " "] {
            let result = parse(&RawCommand::serial(input));
            assert!(result.is_err(), "Input `{}` should fail", input);
        }
    }

    #[test]
    fn parse_invalid_command_name_should_fail() {
        for input in ["_test", "9test", "*", "💩"] {
            let result = parse(&RawCommand::serial(input));
            assert!(
                result.is_err(),
                "Invalid command `{}` was parsed successfully",
                input
            );
        }
    }

    #[test]
    fn parse_valid_command_name_should_succeed() {
        for input in ["echo", "print_state", "disable_motor_1"] {
            let result = parse(&RawCommand::serial(input));
            let parsed = result.expect(&format!("Command `{}` should parse", input));
            assert_eq!(parsed.name, input);
        }
    }

    #[test]
    fn parse_invalid_argument_name_should_fail() {
        for input in ["echo _a", "echo 9test", "echo *", "echo aaa💩"] {
            let result = parse(&RawCommand::serial(input));
            assert!(
                result.is_err(),
                "Input `{}` should fail due to invalid arg name",
                input
            );
        }
    }

    #[test]
    fn parse_invalid_argument_value_should_fail() {
        for input in [
            "echo a=_a",
            "echo a=1💩",
            "echo a=a=",
            "echo a==",
            "echo aaa💩",
        ] {
            let result = parse(&RawCommand::serial(input));
            assert!(
                result.is_err(),
                "Input `{}` should fail due to invalid arg value",
                input
            );
        }
    }

    #[test]
    fn parse_valid_argument_list_should_be_parsed() {
        let inputs = vec![
            ("command argument", vec![("argument", None)]),
            ("command argument=value", vec![("argument", Some("value"))]),
            ("command argument1=10", vec![("argument1", Some("10"))]),
            (
                "command argument1=value1 argument2=value2",
                vec![("argument1", Some("value1")), ("argument2", Some("value2"))],
            ),
            (
                "command a1 a2 a3 a4 a5 a6 a7",
                vec![
                    ("a1", None),
                    ("a2", None),
                    ("a3", None),
                    ("a4", None),
                    ("a5", None),
                    ("a6", None),
                    ("a7", None),
                ],
            ),
        ];

        for (input, expected_args) in inputs {
            let result = parse(&RawCommand::serial(input));
            let parsed = result.expect(&format!("Parsing `{}` failed", input));
            let actual_args = &parsed.args.items;

            assert_eq!(
                actual_args.len(),
                expected_args.len(),
                "Expected {} args, got {} for `{}`",
                expected_args.len(),
                actual_args.len(),
                input
            );

            for (actual, (expected_name, expected_value)) in
                actual_args.iter().zip(expected_args.iter())
            {
                assert_eq!(
                    &actual.name, expected_name,
                    "Arg name mismatch in `{}`",
                    input
                );
                assert_eq!(
                    &actual.value, expected_value,
                    "Arg value mismatch in `{}`",
                    input
                );
            }
        }
    }
}
