use crate::Motor;
use crate::state::{
    CalibratingCurrentSensorState, InitializationState, MotorState, MotorStateEnvelope,
};
use defmt::debug;
use embassy_time::Duration;
use hardware_abstraction::motor_driver::MotorDriver;

const INITIAL_DEAD_TIME: Duration = Duration::from_millis(500);
const CURRENT_SENSOR_PHASE_CALIBRATION_TIME: Duration = Duration::from_millis(100);

pub async fn on_tick(motor: &Motor, driver: &mut impl MotorDriver) {
    let mut envelope = motor.state.lock().await;
    let (state, set_at) = (envelope.state, envelope.state_set_at);
    let duration_since_state_change = set_at.elapsed();

    let mut update_state = |state: MotorState| {
        update_state(state, driver);
        *envelope = MotorStateEnvelope::new(state);
    };

    match state {
        MotorState::NotInitialized => {
            if duration_since_state_change > INITIAL_DEAD_TIME {
                debug!("Starting current calibration");
                update_state(MotorState::Initializing(
                    InitializationState::CalibratingCurrentSensor(
                        CalibratingCurrentSensorState::PhaseAPowered,
                    ),
                ));
                debug!("Phase A powered");
            }
        }
        MotorState::Initializing(state) => match state {
            InitializationState::CalibratingCurrentSensor(state) => {
                if duration_since_state_change > CURRENT_SENSOR_PHASE_CALIBRATION_TIME {
                    match state {
                        CalibratingCurrentSensorState::PhaseAPowered => {
                            update_state(MotorState::Initializing(
                                InitializationState::CalibratingCurrentSensor(
                                    CalibratingCurrentSensorState::PhaseBPowered,
                                ),
                            ));
                            debug!("Phase B powered");
                        }
                        CalibratingCurrentSensorState::PhaseBPowered => {
                            update_state(MotorState::Initializing(
                                InitializationState::CalibratingCurrentSensor(
                                    CalibratingCurrentSensorState::PhaseCPowered,
                                ),
                            ));
                            debug!("Phase C powered");
                        }
                        CalibratingCurrentSensorState::PhaseCPowered => {
                            debug!("Current calibration finished");
                            update_state(MotorState::Idle)
                        }
                    }
                }
            }
        },
        MotorState::Idle => {}
    }
}

fn update_state(new_state: MotorState, driver: &mut impl MotorDriver) {
    match new_state {
        MotorState::NotInitialized => {}
        MotorState::Initializing(state) => match state {
            InitializationState::CalibratingCurrentSensor(state) => match state {
                CalibratingCurrentSensorState::PhaseAPowered => {
                    driver.disable();
                    driver.enable_phase_a();
                    driver.set_voltage_a(0);
                }
                CalibratingCurrentSensorState::PhaseBPowered => {
                    driver.disable();
                    driver.enable_phase_b();
                    driver.set_voltage_b(0);
                }
                CalibratingCurrentSensorState::PhaseCPowered => {
                    driver.disable();
                    driver.enable_phase_c();
                    driver.set_voltage_c(0);
                }
            },
        },
        MotorState::Idle => {
            driver.disable();
        }
    }
}
