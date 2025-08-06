use crate::CommandResult;
use crate::command::Command;
use foc::Motor;
use foc::state::ControlCommand;

pub async fn execute_command(command: Command, motor: &Motor) -> CommandResult {
    match command {
        Command::Echo => handle_echo(),
        Command::GetState => handle_get_state(motor).await,
        Command::SetControlCommand(control_command) => {
            handle_set_control_state(control_command, motor).await
        }
    }
}

fn handle_echo() -> CommandResult {
    CommandResult::Echo
}

async fn handle_get_state(motor: &Motor) -> CommandResult {
    CommandResult::State(motor.snapshot().await)
}

async fn handle_set_control_state(control_command: ControlCommand, motor: &Motor) -> CommandResult {
    let mut command_ptr = motor.command.lock().await;
    *command_ptr = control_command;
    CommandResult::Ok
}
