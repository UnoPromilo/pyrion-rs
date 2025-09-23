use crate::Motor;
use crate::internal_functions::{clarke_transformation, park_transformation};
use crate::state::{
    CalibratingCurrentSensorState::*, ControlCommand, InitializationState::*, MotorState,
    MotorState::*, MotorStateSnapshot, Powered::*, ShaftCalibrationState::*,
};
use embassy_time::{Duration, Ticker};
use fixed::types::I32F32;
use hardware_abstraction::motor_driver::MotorDriver;
use shared::info;
use shared::units::angle::Electrical;
use shared::units::{Angle, Current, Velocity};

const LOOP_FREQUENCY: Duration = Duration::from_hz(1_000);
const STATE_LOOP_DIVIDER: u32 = 40; // 500Hz
const PID_LOOP_DIVIDER: u32 = 10; // 2000Hz

// TODO have single constant for state machine and motor
const ENCODER_CALIBRATION_SPEED_SLOW_RPM: i16 = 100;
const ENCODER_CALIBRATION_SPEED_FAST_RPM: i16 = 300;

pub async fn state_machine_task(motor: &Motor, driver: &mut impl MotorDriver) {
    let mut ticker = Ticker::every(LOOP_FREQUENCY);
    let mut state_counter = 0;
    let mut pid_counter = 0;
    loop {
        ticker.next().await;
        state_counter += 1;
        pid_counter += 1;
        if state_counter == STATE_LOOP_DIVIDER {
            state_loop(motor, driver).await;
            state_counter = 0;
        }
        if pid_counter == PID_LOOP_DIVIDER {
            pid_counter = 0;
            pid_loop(motor).await;
        }
        drive_motor(motor, driver).await;
    }
}

async fn pid_loop(motor: &Motor) {
    let state = {
        let state_snapshot = motor.state.lock().await;
        state_snapshot.state.clone()
    };

    match state {
        Uninitialized => {}
        Initializing(_) => {}
        Idle => {}
        Powered(powered) => match powered {
            ShaftCalibration(_) => update_velocity(motor).await,
        },
    }
}

async fn update_velocity(motor: &Motor) {
    let shaft = { motor.shaft.lock().await.clone() };

    // Can't do anything if we don't have a shaft
    let shaft = match shaft {
        None => return,
        Some(shaft) => shaft,
    };

    // TODO rename all elec. angle to theta?
    let ref_i_q = {
        let raw_velocity = I32F32::from_num(shaft.filtered_velocity.raw());
        // TODO add option to select if it should be raw Velocity, filtered Velocity or estimated Velocity
        let output = motor
            .velocity_pid
            .try_lock()
            .expect("Could not lock Velocity PID, is it already running?")
            .update(raw_velocity)
            .to_num::<i32>();

        Current::from_milliamps(output)
    };
    let ref_i_d = Current::from_milliamps(0);
    {
        // TODO PID should accept Current and Velocity as inputs
        motor
            .i_d_pid
            .try_lock()
            .expect("Could not lock i_d PID, is it already running?")
            .set_target(I32F32::from_num(ref_i_d.as_milliamps()));
        motor
            .i_q_pid
            .try_lock()
            .expect("Could not lock i_q PID, is it already running?")
            .set_target(I32F32::from_num(ref_i_q.as_milliamps()));
    }
}

async fn drive_motor(motor: &Motor, driver: &mut impl MotorDriver) {
    let currents = { motor.current.lock().await.clone() };
    let shaft = { motor.shaft.lock().await.clone() };
    // Can't do anything if we don't have a shaft
    let theta = match shaft {
        None => return,
        Some(shaft) => shaft.estimate_electrical_angle_now(),
    };

    // TODO if Current are none, convert Current to voltage using the constant scaling factor
    let currents = match currents {
        None => return, // todo Drive motor with voltage

        Some(currents) => currents,
    };

    // TODO run assume_balanced when only two currents are given
    let (i_alpha, i_beta) = clarke_transformation::full(
        currents.a.as_milliamps(),
        currents.b.as_milliamps(),
        currents.c.as_milliamps(),
    );

    let (i_d, i_q) = park_transformation::forward(i_alpha, i_beta, &theta);
    let (v_d, v_q): (i32, i32) = {
        (
            motor
                .i_d_pid
                .try_lock()
                .expect("Could not lock i_d PID, is it already running?")
                .update(I32F32::from_num(i_d))
                .to_num(),
            motor
                .i_q_pid
                .try_lock()
                .expect("Could not lock i_q PID, is it already running?")
                .update(I32F32::from_num(i_q))
                .to_num(),
        )
    };
    // TODO Cross-coupling & feedforward (when inductance is known)

    // TODO limit voltage to max voltage (find what it is)
    let (v_alpha, v_beta) = park_transformation::inverse(v_d, v_q, &theta);

    // TODO space vector modulation
    let (v_a, v_b, v_c) = clarke_transformation::inverse(v_alpha, v_beta);

    // TODO do better clamping/conversion logic
    let v_a = v_a.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    let v_b = v_b.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    let v_c = v_c.clamp(i16::MIN as i32, i16::MAX as i32) as i16;

    driver.set_voltages(v_a, v_b, v_c);
}

async fn state_loop(motor: &Motor, driver: &mut impl MotorDriver) {
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
        apply_state_transition(next_state, state_snapshot.state, motor, driver);
        *state_snapshot = MotorStateSnapshot::new(next_state);
    }
}

fn next_motor_state(current: MotorState, elapsed: Duration) -> Option<MotorState> {
    const INITIAL_DEAD_TIME: Duration = Duration::from_millis(500);
    const CURRENT_SENSOR_PHASE_CALIBRATION_TIME: Duration = Duration::from_millis(100);
    const ENCODER_CALIBRATION_STEP_TIME: Duration = Duration::from_millis(1000);

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
        Powered(ShaftCalibration(measurement)) if elapsed > ENCODER_CALIBRATION_STEP_TIME => {
            match measurement {
                WarmUp() => Some(Powered(ShaftCalibration(MeasuringSlow()))),
                MeasuringSlow() => {
                    info!("Slow measurement is done!");
                    info!("Now a little faster");
                    Some(Powered(ShaftCalibration(ChangeSpeed())))
                }
                ChangeSpeed() => Some(Powered(ShaftCalibration(MeasuringFast()))),
                MeasuringFast() => {
                    info!("Measuring done!");
                    info!("Spinning back to the original position");
                    Some(Idle)
                }
            }
        }
        Powered(ShaftCalibration(_)) => None,
    }
}

fn apply_state_transition(
    new_state: MotorState,
    current_state: MotorState,
    motor: &Motor,
    driver: &mut impl MotorDriver,
) {
    match new_state {
        Uninitialized => {}
        Initializing(CalibratingCurrentSensor(phase)) => match phase {
            PhaseAPowered => {
                info!("Calibrating Current sensor, phase A powered");
                driver.disable();
                driver.enable_phase_a();
                driver.set_voltage_a(0);
            }
            PhaseBPowered => {
                info!("Calibrating Current sensor, phase B powered");
                driver.disable();
                driver.enable_phase_b();
                driver.set_voltage_b(0);
            }
            PhaseCPowered => {
                info!("Calibrating Current sensor, phase C powered");
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
                ShaftCalibration(measurement) => match measurement {
                    WarmUp() => {
                        let mut pid = motor
                            .velocity_pid
                            .try_lock()
                            .expect("Could not lock Velocity PID, is it already running?");

                        pid.reset();
                        pid.set_target(I32F32::from_num(
                            Velocity::<Electrical>::from_rpm(ENCODER_CALIBRATION_SPEED_SLOW_RPM)
                                .raw(),
                        ));
                    }
                    MeasuringSlow() => {
                        info!("Measuring slow...");
                    }
                    ChangeSpeed() => {
                        let mut pid = motor
                            .velocity_pid
                            .try_lock()
                            .expect("Could not lock Velocity PID, is it already running?");

                        pid.reset();
                        pid.set_target(I32F32::from_num(
                            Velocity::<Electrical>::from_rpm(ENCODER_CALIBRATION_SPEED_FAST_RPM)
                                .raw(),
                        ));
                    }
                    MeasuringFast() => {
                        info!("Measuring fast...");
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
        Powered(ShaftCalibration(_)) => false,
    }
}

fn handle_command(control_command: ControlCommand) -> Option<MotorState> {
    match control_command {
        ControlCommand::CalibrateShaft => {
            info!("Calibrating shaft");
            info!("Warming up motor");
            Some(Powered(ShaftCalibration(WarmUp())))
        }
        ControlCommand::SetTargetZero => todo!(),
        ControlCommand::SetTargetVoltage(_) => todo!(),
        ControlCommand::SetTargetTorque(_) => todo!(),
        ControlCommand::SetTargetVelocity(_) => todo!(),
        ControlCommand::SetTargetPosition(_) => todo!(),
    }
}
