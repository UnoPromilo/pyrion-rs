use crate::CommandResult;
use crate::command::{Command, Pid, PidValue};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use fixed::types::I32F32;
use foc::Motor;
use foc::state::ControlCommand;
use shared::utils::pid::Controller;

pub async fn execute_command(command: Command, motor: &Motor) -> CommandResult {
    match command {
        Command::Echo => handle_echo(),
        Command::GetState => handle_get_state(motor).await,
        Command::GetShaft => handle_get_shaft(motor).await,
        Command::SetControlCommand(control_command) => {
            handle_set_control_command(control_command, motor).await
        }
        Command::SetPid(pid, values) => handle_set_pid(pid, values, motor).await,
    }
}

fn handle_echo() -> CommandResult {
    CommandResult::Echo
}

async fn handle_get_state(motor: &Motor) -> CommandResult {
    CommandResult::State(motor.snapshot().await)
}

async fn handle_get_shaft(motor: &Motor) -> CommandResult {
    CommandResult::Shaft(motor.snapshot().await.shaft)
}

async fn handle_set_control_command(
    control_command: ControlCommand,
    motor: &Motor,
) -> CommandResult {
    motor.command.signal(control_command);
    CommandResult::Ok
}

async fn handle_set_pid(pid: Pid, values: PidValue, motor: &Motor) -> CommandResult {
    let controllers: &[&Mutex<CriticalSectionRawMutex, Controller<I32F32>>] = match pid {
        Pid::Velocity => &[&motor.velocity_pid],
        Pid::Current => &[&motor.i_q_pid, &motor.i_d_pid],
    };

    for ctrl in controllers {
        let mut controller = ctrl.lock().await;
        controller.set_p(values.kp, values.kp_limit);
        controller.set_i(values.ki, values.ki_limit);
        controller.set_d(values.kd, values.kd_limit);
    }

    CommandResult::Ok
}
