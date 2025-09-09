use defmt::Formatter;
use fixed::types::I32F32;
use foc::MotorSnapshot;
use foc::state::ShaftData;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    Echo,
    GetState,
    GetShaft,
    SetPid(Pid, PidValue),
    SetControlCommand(foc::state::ControlCommand),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommandResult {
    Echo,
    State(MotorSnapshot),
    Shaft(Option<ShaftData>),
    Ok,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pid {
    Velocity,
    Current,
}

pub struct PidValue {
    pub kp: I32F32,
    pub kp_limit: Option<I32F32>,
    pub ki: I32F32,
    pub ki_limit: Option<I32F32>,
    pub kd: I32F32,
    pub kd_limit: Option<I32F32>,
}

impl Default for PidValue {
    fn default() -> Self {
        Self {
            kp: I32F32::from_num(1),
            kp_limit: None,
            ki: I32F32::from_num(0),
            ki_limit: None,
            kd: I32F32::from_num(0),
            kd_limit: None,
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for PidValue {
    fn format(&self, fmt: Formatter) {
        defmt::write!(
            fmt,
            "Kp: {:?}, Ki: {:?}, Kd: {:?}",
            self.kp.to_num::<f32>(),
            self.ki.to_num::<f32>(),
            self.kd.to_num::<f32>(),
        )
    }
}
