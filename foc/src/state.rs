use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant};
use fixed::types::I32F32;
use shared::units::angle::{AngleAny, Electrical, Mechanical};
use shared::units::{Angle, Current, Velocity, Voltage};
use shared::utils::pid;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct PhaseCurrent {
    pub a: Current,
    pub b: Current,
    pub c: Current,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct ShaftData {
    pub angle: AngleAny,
    pub electrical_angle: Angle<Electrical>,
    pub measure_time: Instant,
    pub shaft_calibration: ShaftCalibrationConstants,
    pub filtered_velocity: Velocity<Electrical>,
}

impl ShaftData {
    pub fn estimate_electrical_angle_now(&self) -> Angle<Electrical> {
        let now = Instant::now();
        let elapsed = now - self.measure_time;
        self.electrical_angle
            .overflowing_add(&(self.filtered_velocity * elapsed))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ShaftCalibrationConstants {
    pub offset: Angle<Electrical>,
    pub pole_pairs: i16,
    pub measurement_delay: Duration,
}

impl Default for ShaftCalibrationConstants {
    fn default() -> Self {
        Self {
            offset: Angle::zero(),
            pole_pairs: 1,
            measurement_delay: Duration::from_secs(0),
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ShaftCalibrationConstants {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ShaftCalibrationConstants {{ pole_pairs: {}, offset: {}, measurement_delay: {}μs }}",
            self.pole_pairs,
            self.offset,
            self.measurement_delay.as_micros(),
        );
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct MotorStateSnapshot {
    pub state: MotorState,
    pub state_set_at: Instant,
}

impl MotorStateSnapshot {
    pub fn new(state: MotorState) -> Self {
        Self {
            state,
            state_set_at: Instant::now(),
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum MotorState {
    Uninitialized,
    Initializing(InitializationState),
    Idle,
    Powered(Powered),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum InitializationState {
    CalibratingCurrentSensor(CalibratingCurrentSensorState),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum CalibratingCurrentSensorState {
    PhaseAPowered,
    PhaseBPowered,
    PhaseCPowered,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum Powered {
    ShaftCalibration(ShaftCalibrationState),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum ShaftCalibrationState {
    WarmUp(),
    MeasuringSlow(),
    ChangeSpeed(),
    MeasuringFast(),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum ControlCommand {
    CalibrateShaft,
    SetTargetZero,
    SetTargetVoltage(Voltage),
    SetTargetTorque(Current),
    SetTargetVelocity(Velocity<Mechanical>),
    SetTargetPosition(Angle<Mechanical>),
}

pub struct Motor {
    pub current: Mutex<CriticalSectionRawMutex, Option<PhaseCurrent>>,
    pub shaft: Mutex<CriticalSectionRawMutex, Option<ShaftData>>,
    pub state: Mutex<CriticalSectionRawMutex, MotorStateSnapshot>,
    pub command: Signal<CriticalSectionRawMutex, ControlCommand>,

    // TODO pid values should be configurable and stored in flash, the same as the shaft calibration constants
    // PID controllers should not be used outside the motor state machine
    pub position_pid: Mutex<CriticalSectionRawMutex, pid::Controller<I32F32>>,
    pub velocity_pid: Mutex<CriticalSectionRawMutex, pid::Controller<I32F32>>,
    pub i_d_pid: Mutex<CriticalSectionRawMutex, pid::Controller<I32F32>>,
    pub i_q_pid: Mutex<CriticalSectionRawMutex, pid::Controller<I32F32>>,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct MotorSnapshot {
    pub current: Option<PhaseCurrent>,
    pub shaft: Option<ShaftData>,
    pub state: MotorStateSnapshot,
    pub command: Option<ControlCommand>,
}

impl Motor {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
            shaft: Mutex::new(None),
            state: Mutex::new(MotorStateSnapshot::new(MotorState::Uninitialized)),
            command: Signal::new(),
            position_pid: Mutex::new(pid::Controller::default()),
            velocity_pid: Mutex::new(pid::Controller::default()),
            i_d_pid: Mutex::new(pid::Controller::default()),
            i_q_pid: Mutex::new(pid::Controller::default()),
        }
    }

    pub async fn snapshot(&self) -> MotorSnapshot {
        let copied_command = {
            let value = self.command.try_take();
            if let Some(value) = value {
                self.command.signal(value);
            }
            value
        };
        MotorSnapshot {
            current: self.current.lock().await.clone(),
            shaft: self.shaft.lock().await.clone(),
            state: self.state.lock().await.clone(),
            command: copied_command,
        }
    }
}
