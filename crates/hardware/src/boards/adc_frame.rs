#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase<T> {
    pub u: T,
    pub v: T,
    pub w: T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FastAdcCounts {
    pub current: Phase<u16>,
    pub voltage: Phase<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuxiliaryAdcCounts {
    pub motor_temperature: u16,
    pub mosfet_temperature: u16,
    pub bus_voltage: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardAdcFrame {
    pub fast: FastAdcCounts,
}

impl BoardAdcFrame {
    pub fn from_injected(adc1: [u16; 2], adc3: [u16; 2], adc5: [u16; 2]) -> Self {
        let [i_u, v_u] = adc1;
        let [i_v, v_v] = adc3;
        let [i_w, v_w] = adc5;

        Self {
            fast: FastAdcCounts {
                current: Phase {
                    u: i_u,
                    v: i_v,
                    w: i_w,
                },
                voltage: Phase {
                    u: v_u,
                    v: v_v,
                    w: v_w,
                },
            },
        }
    }
}

impl AuxiliaryAdcCounts {
    pub fn from_regular(adc2: [u16; 3]) -> Self {
        let [mosfet_temperature, motor_temperature, bus_voltage] = adc2;
        Self {
            motor_temperature,
            mosfet_temperature,
            bus_voltage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_rank_to_its_board_signal() {
        let frame = BoardAdcFrame::from_injected([101, 102], [301, 302], [501, 502]);
        assert_eq!(
            frame.fast.current,
            Phase {
                u: 101,
                v: 301,
                w: 501
            }
        );
        assert_eq!(
            frame.fast.voltage,
            Phase {
                u: 102,
                v: 302,
                w: 502
            }
        );
        assert_eq!(
            AuxiliaryAdcCounts::from_regular([201, 202, 203]),
            AuxiliaryAdcCounts {
                mosfet_temperature: 201,
                motor_temperature: 202,
                bus_voltage: 203,
            }
        );
    }
}
