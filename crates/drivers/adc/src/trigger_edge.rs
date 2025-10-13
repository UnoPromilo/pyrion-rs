use stm32_metapac::adc::vals::Exten;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ExtTriggerEdge {
    Rising,
    Falling,
    Both,
}

impl From<ExtTriggerEdge> for Exten {
    fn from(value: ExtTriggerEdge) -> Self {
        match value {
            ExtTriggerEdge::Rising => Exten::RISING_EDGE,
            ExtTriggerEdge::Falling => Exten::FALLING_EDGE,
            ExtTriggerEdge::Both => Exten::BOTH_EDGES,
        }
    }
}
