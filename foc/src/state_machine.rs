use crate::Motor;
use crate::state::{
    CalibratingCurrentSensorState::*, InitializationState::*, MotorState, MotorState::*,
    MotorStateSnapshot,
};
use defmt::info;
use embassy_time::Duration;
use hardware_abstraction::motor_driver::MotorDriver;

pub async fn on_tick(motor: &Motor, driver: &mut impl MotorDriver) {
    let mut state_snapshot = motor.state.lock().await;
    let duration = state_snapshot.state_set_at.elapsed();

    if let Some(next_state) = next_motor_state(state_snapshot.state, duration) {
        apply_state_transition(next_state, driver);
        *state_snapshot = MotorStateSnapshot::new(next_state);
    }
}

fn next_motor_state(current: MotorState, elapsed: Duration) -> Option<MotorState> {
    const INITIAL_DEAD_TIME: Duration = Duration::from_millis(500);
    const CURRENT_SENSOR_PHASE_CALIBRATION_TIME: Duration = Duration::from_millis(100);

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
        }
    }
}
