use crate::Motor;
use crate::state::{
    CalibratingCurrentSensorState::*, ControlCommand, InitializationState::*, MeasurementState::*,
    MotorState, MotorState::*, MotorStateSnapshot,
};
use embassy_time::Duration;
use hardware_abstraction::motor_driver::MotorDriver;
use shared::info;
use shared::units::Angle;
use shared::units::angle::Electrical;

pub async fn on_tick(motor: &Motor, driver: &mut impl MotorDriver) {
    let mut state_snapshot = motor.state.lock().await;
    let duration = state_snapshot.state_set_at.elapsed();

    let next_motor_state = if does_state_allow_command_handling(state_snapshot.state)
        && let Some(command) = motor.command.try_take()
        && let Some(new_state) = handle_command(command)
    {
        Some(new_state)
    } else {
        next_motor_state(state_snapshot.state, duration)
    };

    if let Some(next_state) = next_motor_state {
        apply_state_transition(next_state, driver);
        *state_snapshot = MotorStateSnapshot::new(next_state);
    }
}

fn next_motor_state(current: MotorState, elapsed: Duration) -> Option<MotorState> {
    const INITIAL_DEAD_TIME: Duration = Duration::from_millis(500);
    const CURRENT_SENSOR_PHASE_CALIBRATION_TIME: Duration = Duration::from_millis(100);
    const MEASUREMENT_STEP_TIME: Duration = Duration::from_millis(10);
    const ENCODER_CALIBRATION_STEP: Angle<Electrical> = Angle::<Electrical>::from_raw(364); // Around 2 degrees
    const ENCODER_POLES_MEASURING_ROTATIONS: u8 = 3;

    match current {
        Uninitialized if elapsed > INITIAL_DEAD_TIME => {
            Some(Initializing(CalibratingCurrentSensor(PhaseAPowered)))
        }
        Uninitialized => None,
        Initializing(CalibratingCurrentSensor(phase))
            if elapsed > CURRENT_SENSOR_PHASE_CALIBRATION_TIME =>
        {
            match phase {
                PhaseAPowered => Some(Initializing(CalibratingCurrentSensor(PhaseBPowered))),
                PhaseBPowered => Some(Initializing(CalibratingCurrentSensor(PhaseCPowered))),
                PhaseCPowered => Some(Idle),
            }
        }
        Initializing(CalibratingCurrentSensor(_)) => None,
        Idle => None,
        Measuring(measurement) if elapsed > MEASUREMENT_STEP_TIME => match measurement {
            Direction(current_angle) => {
                if let Some(new_angle) = current_angle.checked_add(&ENCODER_CALIBRATION_STEP) {
                    Some(Measuring(Direction(new_angle)))
                } else {
                    info!("Measuring direction done");
                    info!("Measuring magnetic poles");
                    Some(Measuring(MagneticPoles(
                        Angle::<Electrical>::from_raw(0),
                        0,
                    )))
                }
            }
            MagneticPoles(current_angle, rotation_count) => {
                if let Some(new_angle) = current_angle.checked_add(&ENCODER_CALIBRATION_STEP) {
                    Some(Measuring(MagneticPoles(new_angle, rotation_count)))
                } else if rotation_count < ENCODER_POLES_MEASURING_ROTATIONS {
                    let new_angle = current_angle.overflowing_add(&ENCODER_CALIBRATION_STEP);
                    Some(Measuring(MagneticPoles(new_angle, rotation_count + 1)))
                } else {
                    info!("Measuring magnetic poles done");
                    info!("Measuring magnetic offset");
                    Some(Measuring(MagneticOffset(Angle::<Electrical>::from_raw(0))))
                }
            }
            MagneticOffset(current_angle) => {
                if let Some(new_angle) = current_angle.checked_add(&ENCODER_CALIBRATION_STEP) {
                    Some(Measuring(MagneticOffset(new_angle)))
                } else {
                    info!("Measuring magnetic offset done");
                    info!("Idle");
                    Some(Idle)
                }
            }
        },
        Measuring(_) => None,
    }
}

fn apply_state_transition(new_state: MotorState, driver: &mut impl MotorDriver) {
    match new_state {
        Uninitialized => {}
        Initializing(CalibratingCurrentSensor(phase)) => match phase {
            PhaseAPowered => {
                info!("Calibrating current sensor, phase A powered");
                driver.disable();
                driver.enable_phase_a();
                driver.set_voltage_a(0);
            }
            PhaseBPowered => {
                info!("Calibrating current sensor, phase B powered");
                driver.disable();
                driver.enable_phase_b();
                driver.set_voltage_b(0);
            }
            PhaseCPowered => {
                info!("Calibrating current sensor, phase C powered");
                driver.disable();
                driver.enable_phase_c();
                driver.set_voltage_c(0);
            }
        },
        Idle => {
            info!("Idle");
            driver.disable();
            driver.enable_phase_a();
            driver.set_voltage_a(10);
        }
        Measuring(measurement) => match measurement {
            Direction(angle) | MagneticPoles(angle, _) | MagneticOffset(angle) => {
                drive_motor(angle);
            }
        },
    }
}

fn does_state_allow_command_handling(current: MotorState) -> bool {
    match current {
        Uninitialized => false,
        Initializing(_) => false,
        Idle => true,
        Measuring(_) => false,
    }
}

fn handle_command(control_command: ControlCommand) -> Option<MotorState> {
    match control_command {
        ControlCommand::CalibrateEncoder => {
            info!("Calibrating encoder");
            info!("Measuring magnetic direction");
            Some(Measuring(Direction(Angle::<Electrical>::from_raw(0))))
        }
        ControlCommand::SetTargetZero => todo!(),
        ControlCommand::SetTargetVoltage(_) => todo!(),
        ControlCommand::SetTargetTorque(_) => todo!(),
        ControlCommand::SetTargetVelocity(_) => todo!(),
        ControlCommand::SetTargetPosition(_) => todo!(),
    }
}

fn drive_motor(angle: Angle<Electrical>) {
    todo!()
}
