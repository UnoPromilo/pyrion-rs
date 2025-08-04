use foc::Motor;
use crate::CommandResult;
use crate::command::Command;

pub async fn execute_command(command: &Command, motor: &Motor) -> CommandResult {
    match command {
        Command::Echo => handle_echo(),
        Command::GetState => handle_get_state(motor).await,
    }
}

fn handle_echo() -> CommandResult {
    CommandResult::Echo
}

async fn handle_get_state(motor: &Motor) -> CommandResult {
    CommandResult::State(motor.freeze().await)
}
