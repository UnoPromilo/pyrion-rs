use crate::Motor;
use crate::internal_functions::{clarke_transformation, park_transformation};
use crate::state::{
    CalibratingCurrentSensorState::*, ControlCommand, EncoderCalibrationState::*,
    InitializationState::*, MotorState, MotorState::*, MotorStateSnapshot, Powered::*,
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
        apply_state_transition(next_state, state_snapshot.state, driver);
        *state_snapshot = MotorStateSnapshot::new(next_state);
    }
}

fn next_motor_state(current: MotorState, elapsed: Duration) -> Option<MotorState> {
    const INITIAL_DEAD_TIME: Duration = Duration::from_millis(500);
    const CURRENT_SENSOR_PHASE_CALIBRATION_TIME: Duration = Duration::from_millis(100);
    const MEASUREMENT_STEP_TIME: Duration = Duration::from_hz(2000);

    // TODO have single constant for state machine and motor
    const ENCODER_CALIBRATION_STEP_SLOW: Angle<Electrical> = Angle::<Electrical>::from_raw(128); // Around 2 degrees
    const ENCODER_CALIBRATION_STEPS_FAST: Angle<Electrical> = Angle::<Electrical>::from_raw(512);
    const ENCODER_POLES_MEASURING_ROTATIONS: u8 = 14;

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
        Powered(EncoderCalibration(measurement)) if elapsed > MEASUREMENT_STEP_TIME => {
            match measurement {
                WarmUp(current_angle) => {
                    if let Some(new_angle) =
                        current_angle.checked_add(&ENCODER_CALIBRATION_STEP_SLOW)
                    {
                        Some(Powered(EncoderCalibration(WarmUp(new_angle))))
                    } else {
                        info!("Measuring...");
                        Some(Powered(EncoderCalibration(MeasuringSlow(
                            Angle::<Electrical>::zero(),
                            0,
                        ))))
                    }
                }
                MeasuringSlow(current_angle, rotation_count) => {
                    if let Some(new_angle) =
                        current_angle.checked_add(&ENCODER_CALIBRATION_STEP_SLOW)
                    {
                        Some(Powered(EncoderCalibration(MeasuringSlow(
                            new_angle,
                            rotation_count,
                        ))))
                    } else if rotation_count < ENCODER_POLES_MEASURING_ROTATIONS {
                        let new_angle =
                            current_angle.overflowing_add(&ENCODER_CALIBRATION_STEP_SLOW);
                        Some(Powered(EncoderCalibration(MeasuringSlow(
                            new_angle,
                            rotation_count + 1,
                        ))))
                    } else {
                        let new_angle =
                            current_angle.overflowing_add(&ENCODER_CALIBRATION_STEPS_FAST);
                        info!("Slow measurement is done!");
                        info!("Now a little faster");
                        Some(Powered(EncoderCalibration(MeasuringFast(new_angle, 0))))
                    }
                }
                MeasuringFast(current_angle, rotation_count) => {
                    if let Some(new_angle) =
                        current_angle.checked_add(&ENCODER_CALIBRATION_STEPS_FAST)
                    {
                        Some(Powered(EncoderCalibration(MeasuringFast(
                            new_angle,
                            rotation_count,
                        ))))
                    } else if rotation_count < ENCODER_POLES_MEASURING_ROTATIONS {
                        let new_angle =
                            current_angle.overflowing_add(&ENCODER_CALIBRATION_STEPS_FAST);
                        Some(Powered(EncoderCalibration(MeasuringFast(
                            new_angle,
                            rotation_count + 1,
                        ))))
                    } else {
                        info!("Measuring done!");
                        info!("Spinning back to the original position");
                        Some(Powered(EncoderCalibration(Return(
                            Angle::<Electrical>::max(),
                            ENCODER_POLES_MEASURING_ROTATIONS * 2 + 1,
                        ))))
                    }
                }
                Return(current_angle, rotation_count) => {
                    if let Some(new_angle) =
                        current_angle.checked_sub(&ENCODER_CALIBRATION_STEPS_FAST)
                    {
                        Some(Powered(EncoderCalibration(Return(
                            new_angle,
                            rotation_count,
                        ))))
                    } else if rotation_count > 0 {
                        let new_angle =
                            current_angle.overflowing_sub(&ENCODER_CALIBRATION_STEPS_FAST);
                        Some(Powered(EncoderCalibration(Return(
                            new_angle,
                            rotation_count - 1,
                        ))))
                    } else {
                        Some(Idle)
                    }
                }
            }
        }
        Powered(EncoderCalibration(_)) => None,
    }
}

fn apply_state_transition(
    new_state: MotorState,
    current_state: MotorState,
    driver: &mut impl MotorDriver,
) {
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
        }
        Powered(powered) => {
            if !matches!(current_state, Powered(_)) {
                driver.enable_synced();
            }
            match powered {
                EncoderCalibration(measurement) => match measurement {
                    WarmUp(angle)
                    | MeasuringSlow(angle, _)
                    | MeasuringFast(angle, _)
                    | Return(angle, _) => {
                        drive_motor(&angle, driver);
                    }
                },
            }
        }
    }
}

fn does_state_allow_command_handling(current: MotorState) -> bool {
    match current {
        Uninitialized => false,
        Initializing(_) => false,
        Idle => true,
        Powered(EncoderCalibration(_)) => false,
    }
}

fn handle_command(control_command: ControlCommand) -> Option<MotorState> {
    match control_command {
        ControlCommand::CalibrateShaft => {
            info!("Calibrating shaft");
            info!("Warming up motor");
            Some(Powered(EncoderCalibration(WarmUp(
                Angle::<Electrical>::zero(),
            ))))
        }
        ControlCommand::SetTargetZero => todo!(),
        ControlCommand::SetTargetVoltage(_) => todo!(),
        ControlCommand::SetTargetTorque(_) => todo!(),
        ControlCommand::SetTargetVelocity(_) => todo!(),
        ControlCommand::SetTargetPosition(_) => todo!(),
    }
}

fn drive_motor(angle: &Angle<Electrical>, motor: &mut impl MotorDriver) {
    let (alpha, beta) = park_transformation::inverse(0, i16::MAX / 3, angle);
    let (voltage_a, voltage_b, voltage_c) = clarke_transformation::inverse(alpha, beta);
    motor.set_voltages(voltage_a, voltage_b, voltage_c);
}
